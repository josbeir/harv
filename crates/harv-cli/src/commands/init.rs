use harv_sdk::{Alias, HarvClient, NoteTemplate, PROJECT_CONFIG_FILENAME, ProjectConfig};

use crate::prompts;
use crate::spinner;

/// Run the `harv init` command.
pub async fn run(args: &crate::InitArgs) -> color_eyre::eyre::Result<()> {
    let cwd = std::env::current_dir()?;
    let target_path = cwd.join(PROJECT_CONFIG_FILENAME);

    // Check for existing harv.toml
    if target_path.exists() && !args.force {
        println!(
            "A {} already exists in this directory.",
            PROJECT_CONFIG_FILENAME
        );
        println!("Use --force to overwrite it, or edit it manually.");
        return Ok(());
    }

    // Non-interactive path: all required flags provided
    if args.project_id.is_some() && args.task_id.is_some() {
        return create_non_interactive(args, &target_path).await;
    }

    // Interactive wizard
    let client = HarvClient::from_config_file().await?;
    let config = client.config().clone();

    let (assignments, _) = spinner::with_spinner(
        "Loading project assignments...",
        client.projects().my_assignments(false),
    )
    .await?;

    let choices = prompts::build_project_choices(&assignments, config.last_project_id());
    if choices.is_empty() {
        println!("No project assignments found.");
        return Ok(());
    }

    // Step 1: Pick default project (pre-select last used if available)
    let (project_id, project_name, task_assignments) = if let Some(pid) = args.project_id {
        let choice = choices
            .iter()
            .find(|c| c.project_id == pid)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Project ID {} not found in your assignments", pid)
            })?;
        (pid, choice.display.clone(), choice.task_assignments.clone())
    } else {
        let cursor =
            crate::resolution::starting_cursor_for_default(&choices, config.last_project_id());
        println!("\nSelect the default project for this directory:");
        let choice = prompts::pick_project(&choices, cursor)?;
        (
            choice.project_id,
            choice.display.clone(),
            choice.task_assignments.clone(),
        )
    };

    // Step 2: Pick default task (always prompt — this is a setup wizard)
    let task_id = if let Some(tid) = args.task_id {
        if !task_assignments.iter().any(|t| t.task.id == tid) {
            return Err(color_eyre::eyre::eyre!(
                "Task ID {} not assigned to project {}",
                tid,
                project_id
            ));
        }
        tid
    } else {
        println!("\nSelect the default task for this project:");
        let temp_choice = prompts::ProjectChoice {
            display: project_name,
            project_id,
            task_assignments: task_assignments.clone(),
        };
        prompts::pick_task(&temp_choice)?.task.id
    };

    // Step 3: Set up templates
    let mut templates = std::collections::HashMap::new();
    if !args.template.is_empty() {
        for tpl_str in &args.template {
            let (name, tpl) = parse_template_arg(tpl_str)?;
            templates.insert(name, tpl);
        }
    } else {
        // Interactive: ask if user wants templates
        println!();
        let add_template = inquire::Confirm::new("Set up a note template?")
            .with_default(false)
            .prompt()?;

        if add_template {
            println!();
            println!("Available template variables:");
            println!("  {{date}}, {{time}}, {{hostname}}, {{commit_message}}, {{branch_name}}");
            println!("  {{project_name}}, {{task_name}}, {{user_name}}");
            println!();

            let name = inquire::Text::new("Template name (e.g. \"daily\"):")
                .with_default("default")
                .prompt()?;

            let pattern = inquire::Text::new("Template pattern:")
                .with_help_message("Use {variable} placeholders for dynamic content")
                .prompt()?;

            templates.insert(name, NoteTemplate { pattern });

            // Ask for more
            loop {
                let more = inquire::Confirm::new("Add another template?")
                    .with_default(false)
                    .prompt()?;
                if !more {
                    break;
                }

                let name = inquire::Text::new("Template name:").prompt()?;
                let pattern = inquire::Text::new("Template pattern:").prompt()?;
                templates.insert(name, NoteTemplate { pattern });
            }
        }
    }

    // Step 4: Set up aliases
    let mut aliases = std::collections::HashMap::new();
    if !args.alias.is_empty() {
        for alias_str in &args.alias {
            let (name, alias) = parse_alias_arg(alias_str)?;
            aliases.insert(name, alias);
        }
    } else {
        // Interactive: ask if user wants project aliases
        println!();
        let add_alias = inquire::Confirm::new("Set up project aliases?")
            .with_default(false)
            .prompt()?;

        if add_alias {
            loop {
                println!();
                let alias_name = prompts::prompt_alias_name()?;

                println!("\nSelect project for alias '{}':", alias_name);
                let alias_choice = prompts::pick_project(&choices, 0)?;
                let alias_task = prompts::pick_task(alias_choice)?;

                aliases.insert(
                    alias_name,
                    Alias {
                        project_id: alias_choice.project_id,
                        task_id: alias_task.task.id,
                    },
                );

                let more = inquire::Confirm::new("Add another alias?")
                    .with_default(false)
                    .prompt()?;
                if !more {
                    break;
                }
            }
        }
    }

    // Step 5: Build and save config
    let config = ProjectConfig {
        default_project_id: Some(project_id),
        default_task_id: Some(task_id),
        aliases,
        templates,
    };

    config.save_to(&target_path).await?;

    println!();
    println!("Created {} in {}", PROJECT_CONFIG_FILENAME, cwd.display());
    if config.default_project_id.is_some() {
        println!("  Default project set");
    }
    if !config.templates.is_empty() {
        println!("  {} template(s) configured", config.templates.len());
    }
    if !config.aliases.is_empty() {
        println!("  {} alias(es) configured", config.aliases.len());
    }
    println!();
    println!("You can edit this file at any time to adjust settings.");

    Ok(())
}

/// Non-interactive path: create harv.toml from CLI flags only.
async fn create_non_interactive(
    args: &crate::InitArgs,
    target_path: &std::path::Path,
) -> color_eyre::eyre::Result<()> {
    let mut config = ProjectConfig {
        default_project_id: args.project_id,
        default_task_id: args.task_id,
        ..Default::default()
    };

    for tpl_str in &args.template {
        let (name, tpl) = parse_template_arg(tpl_str)?;
        config.templates.insert(name, tpl);
    }

    for alias_str in &args.alias {
        let (name, alias) = parse_alias_arg(alias_str)?;
        config.aliases.insert(name, alias);
    }

    config.save_to(target_path).await?;

    let cwd = std::env::current_dir()?;
    println!("Created {} in {}", PROJECT_CONFIG_FILENAME, cwd.display());

    Ok(())
}

/// Parse a `--template name=pattern` argument.
fn parse_template_arg(input: &str) -> color_eyre::eyre::Result<(String, NoteTemplate)> {
    let (name, pattern) = input.split_once('=').ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "Invalid template format: '{}'. Expected NAME=PATTERN (e.g. daily=Daily: {{date}})",
            input
        )
    })?;
    if name.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "Template name cannot be empty in '{}'. Use NAME=PATTERN format.",
            input
        ));
    }
    if pattern.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "Template pattern cannot be empty in '{}'. Use NAME=PATTERN format.",
            input
        ));
    }
    Ok((
        name.to_string(),
        NoteTemplate {
            pattern: pattern.to_string(),
        },
    ))
}

/// Parse a `--alias name=project_id:task_id` argument.
fn parse_alias_arg(input: &str) -> color_eyre::eyre::Result<(String, Alias)> {
    let (name, ids) = input.split_once('=').ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "Invalid alias format: '{}'. Expected NAME=PID:TID (e.g. dev=123:456)",
            input
        )
    })?;
    if name.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "Alias name cannot be empty in '{}'. Use NAME=PID:TID format.",
            input
        ));
    }
    let (pid_str, tid_str) = ids
        .split_once(':')
        .ok_or_else(|| color_eyre::eyre::eyre!(
            "Invalid alias format: '{}'. Expected NAME=PID:TID (e.g. dev=123:456), but got '{}' after =",
            input, ids
        ))?;
    let project_id: u64 = pid_str.parse().map_err(|_| {
        color_eyre::eyre::eyre!(
            "Invalid project ID '{}' in alias '{}'. Expected a number.",
            pid_str,
            input
        )
    })?;
    let task_id: u64 = tid_str.parse().map_err(|_| {
        color_eyre::eyre::eyre!(
            "Invalid task ID '{}' in alias '{}'. Expected a number.",
            tid_str,
            input
        )
    })?;
    Ok((
        name.to_string(),
        Alias {
            project_id,
            task_id,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_arg_valid() {
        let (name, tpl) = parse_template_arg("daily=Hello {date}").unwrap();
        assert_eq!(name, "daily");
        assert_eq!(tpl.pattern, "Hello {date}");
    }

    #[test]
    fn test_parse_template_arg_no_equals() {
        let err = parse_template_arg("dailypattern").unwrap_err();
        assert!(err.to_string().contains("NAME=PATTERN"));
    }

    #[test]
    fn test_parse_template_arg_empty_name() {
        let err = parse_template_arg("=pattern").unwrap_err();
        assert!(err.to_string().contains("name cannot be empty"));
    }

    #[test]
    fn test_parse_template_arg_empty_pattern() {
        let err = parse_template_arg("name=").unwrap_err();
        assert!(err.to_string().contains("pattern cannot be empty"));
    }

    #[test]
    fn test_parse_alias_arg_valid() {
        let (name, alias) = parse_alias_arg("dev=100:200").unwrap();
        assert_eq!(name, "dev");
        assert_eq!(alias.project_id, 100);
        assert_eq!(alias.task_id, 200);
    }

    #[test]
    fn test_parse_alias_arg_no_equals() {
        let err = parse_alias_arg("dev100:200").unwrap_err();
        assert!(err.to_string().contains("NAME=PID:TID"));
    }

    #[test]
    fn test_parse_alias_arg_invalid_ids() {
        let err = parse_alias_arg("dev=abc:200").unwrap_err();
        assert!(err.to_string().contains("Invalid project ID"));
    }

    #[test]
    fn test_parse_alias_arg_missing_colon() {
        let err = parse_alias_arg("dev=100").unwrap_err();
        assert!(err.to_string().contains("NAME=PID:TID"));
    }

    #[test]
    fn test_parse_alias_arg_empty_name() {
        let err = parse_alias_arg("=100:200").unwrap_err();
        assert!(err.to_string().contains("name cannot be empty"));
    }
}
