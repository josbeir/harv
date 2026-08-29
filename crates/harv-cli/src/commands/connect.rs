use harv_core::{HarvError, t, t_args};
use harv_sdk::auth;
use harv_sdk::{AuthMethod, Authentication, HarvClient, HarvConfig};
use inquire::{Password, Select, Text, validator::Validation};

#[derive(Clone, Copy)]
enum ConnectMethod {
    QuickOAuth,
    PersonalToken,
    RefreshableOAuth,
}

pub async fn run() -> color_eyre::eyre::Result<()> {
    let method = choose_method()?;
    let config = match method {
        ConnectMethod::QuickOAuth => connect_quick_oauth().await?,
        ConnectMethod::PersonalToken => connect_personal_token().await?,
        ConnectMethod::RefreshableOAuth => connect_refreshable_oauth().await?,
    };
    config.save_authentication().await.map_err(|error| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-connect-save-failed", &[("err", error.user_message())])
        )
    })?;
    println!(
        "{}",
        t_args(
            "cli-connect-success",
            &[("path", HarvConfig::path().display().to_string())]
        )
    );
    Ok(())
}

fn choose_method() -> color_eyre::eyre::Result<ConnectMethod> {
    let quick = t("cli-connect-method-quick");
    let personal = t("cli-connect-method-pat");
    let refreshable = t("cli-connect-method-refreshable");
    Select::new(
        &t("cli-connect-method-prompt"),
        vec![quick.clone(), personal.clone(), refreshable.clone()],
    )
    .with_starting_cursor(0)
    .prompt()
    .map(|choice| {
        if choice == quick {
            ConnectMethod::QuickOAuth
        } else if choice == personal {
            ConnectMethod::PersonalToken
        } else {
            ConnectMethod::RefreshableOAuth
        }
    })
    .map_err(Into::into)
}

async fn connect_quick_oauth() -> color_eyre::eyre::Result<HarvConfig> {
    eprintln!("{}", t("cli-connect-opening"));
    eprintln!("{}\n", t("cli-auth-manual-url"));
    let credentials = auth::authenticate().await.map_err(connect_error)?;
    Ok(config_with_credentials(
        AuthMethod::QuickOAuth,
        credentials.access_token,
        credentials.account_id,
        credentials.expires_at,
        None,
        None,
        None,
    )
    .await?)
}

async fn connect_personal_token() -> color_eyre::eyre::Result<HarvConfig> {
    let token = Password::new(&t("cli-connect-pat-token-prompt"))
        .without_confirmation()
        .prompt()?;
    let account_id = Text::new(&t("cli-connect-pat-account-prompt"))
        .with_validator(|value: &str| {
            Ok(
                if !value.is_empty() && value.chars().all(|character| character.is_ascii_digit()) {
                    Validation::Valid
                } else {
                    Validation::Invalid(t("cli-connect-pat-account-invalid").into())
                },
            )
        })
        .prompt()?;
    let config = config_with_credentials(
        AuthMethod::PersonalAccessToken,
        token,
        account_id,
        None,
        None,
        None,
        None,
    )
    .await?;
    validate_credentials(&config).await.map_err(connect_error)?;
    Ok(config)
}

async fn connect_refreshable_oauth() -> color_eyre::eyre::Result<HarvConfig> {
    let client_id = Text::new(&t("cli-connect-client-id-prompt"))
        .with_validator(non_empty_validator)
        .prompt()?;
    let client_secret = Password::new(&t("cli-connect-client-secret-prompt"))
        .without_confirmation()
        .prompt()?;
    eprintln!("{}", t("cli-connect-opening"));
    eprintln!("{}\n", t("cli-auth-manual-url"));
    let credentials = auth::authenticate_refreshable(&client_id, &client_secret)
        .await
        .map_err(connect_error)?;
    Ok(config_with_credentials(
        AuthMethod::RefreshableOAuth,
        credentials.access_token,
        credentials.account_id,
        credentials.expires_at,
        credentials.refresh_token,
        Some(client_id),
        Some(client_secret),
    )
    .await?)
}

fn non_empty_validator(value: &str) -> Result<Validation, inquire::CustomUserError> {
    Ok(if value.is_empty() {
        Validation::Invalid(t("cli-connect-value-required").into())
    } else {
        Validation::Valid
    })
}

async fn config_with_credentials(
    method: AuthMethod,
    access_token: String,
    account_id: String,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    refresh_token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<HarvConfig, HarvError> {
    let config = match HarvConfig::load().await {
        Ok(config) => config,
        Err(HarvError::ConfigNotFound(_)) => {
            HarvConfig::new(access_token.clone(), account_id.clone())
        }
        Err(error) => return Err(error),
    };
    Ok(apply_authentication(
        config,
        Authentication::new(
            method,
            access_token,
            account_id,
            expires_at,
            refresh_token,
            client_id,
            client_secret,
        ),
    ))
}

fn apply_authentication(mut config: HarvConfig, authentication: Authentication) -> HarvConfig {
    config.set_authentication(authentication);
    config
}

async fn validate_credentials(config: &HarvConfig) -> Result<(), HarvError> {
    HarvClient::new(config.clone())?
        .users()
        .me()
        .await
        .map(|_| ())
}

fn connect_error(error: HarvError) -> color_eyre::eyre::Report {
    color_eyre::eyre::eyre!(
        "{}",
        t_args("cli-connect-failed", &[("err", error.user_message())])
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use harv_sdk::Alias;

    #[test]
    fn applying_personal_token_keeps_existing_preferences() {
        let mut config = HarvConfig::new("old".into(), "1".into());
        config.set_cache_ttl_hours(12);
        config.set_locale(Some("nl".into()));
        config.insert_alias(
            "work",
            Alias {
                project_id: 10,
                task_id: 20,
            },
        );
        let config = apply_authentication(
            config,
            Authentication::new(
                AuthMethod::PersonalAccessToken,
                "personal".into(),
                "2".into(),
                None,
                None,
                None,
                None,
            ),
        );
        assert_eq!(config.access_token(), "personal");
        assert_eq!(config.account_id(), "2");
        assert_eq!(config.auth_method(), AuthMethod::PersonalAccessToken);
        assert_eq!(config.cache_ttl_hours(), 12);
        assert_eq!(config.locale(), Some("nl"));
        assert_eq!(config.alias("work").map(|alias| alias.task_id), Some(20));
    }
}
