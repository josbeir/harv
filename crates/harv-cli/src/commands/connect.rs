use harv_core::{HarvError, t, t_args};
use harv_sdk::auth;
use harv_sdk::{AuthMethod, Authentication, HarvConfig};
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
        ConnectMethod::PersonalToken => connect_personal_token()?,
        ConnectMethod::RefreshableOAuth => connect_refreshable_oauth().await?,
    };
    config.save().await.map_err(|error| {
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

fn connect_personal_token() -> color_eyre::eyre::Result<HarvConfig> {
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
    let mut config = HarvConfig::new(token.clone(), account_id.clone());
    config.set_authentication(Authentication::new(
        AuthMethod::PersonalAccessToken,
        token,
        account_id,
        None,
        None,
        None,
        None,
    ));
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
    let mut config = match HarvConfig::load().await {
        Ok(config) => config,
        Err(HarvError::ConfigNotFound(_)) => {
            HarvConfig::new(access_token.clone(), account_id.clone())
        }
        Err(error) => return Err(error),
    };
    config.set_authentication(Authentication::new(
        method,
        access_token,
        account_id,
        expires_at,
        refresh_token,
        client_id,
        client_secret,
    ));
    Ok(config)
}

fn connect_error(error: HarvError) -> color_eyre::eyre::Report {
    color_eyre::eyre::eyre!(
        "{}",
        t_args("cli-connect-failed", &[("err", error.user_message())])
    )
}
