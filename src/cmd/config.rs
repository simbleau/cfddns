use crate::{
    config::{default_config_path, models::ConfigOpts},
    io::{
        self,
        scanner::{prompt, prompt_ron, prompt_t, prompt_yes_or_no},
    },
};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Configuration controls
#[derive(Debug, Args)]
#[clap(name = "config")]
pub struct ConfigCmd {
    #[clap(subcommand)]
    action: ConfigSubcommands,
}

#[derive(Clone, Debug, Subcommand)]
enum ConfigSubcommands {
    /// Build a configuration file.
    Build,
    /// Show the current configuration.
    Show,
}

impl ConfigCmd {
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        match self.action {
            ConfigSubcommands::Build => build().await,
            ConfigSubcommands::Show => show(config).await,
        }
    }
}

#[tracing::instrument(level = "trace")]
async fn build() -> Result<()> {
    // Prompt
    println!("Welcome! This builder will build a CLI configuration file without needing to understand TOML.");
    println!("For annotated examples of each field, please visit https://github.com/simbleau/cddns/blob/main/config.toml");
    println!("You can skip any field for configuration defaults via enter (no answer.)");
    println!();

    // Build
    let mut builder = ConfigOpts::builder();
    builder
        .verify_token(prompt("Cloudflare API token", "string")?)
        .list_include_zones(prompt_ron(
            "Include zone filters, e.g. `[\".*.com\"]`",
            "list[string]",
        )?)
        .list_ignore_zones(prompt_ron(
            "Ignore zone filters, e.g. `[\"ex1.com\", \"ex2.com\"]`",
            "list[string]",
        )?)
        .list_include_records(prompt_ron(
            "Include record filters, e.g. `[\"shop.imbleau.com\"]`",
            "list[string]",
        )?)
        .list_ignore_records(prompt_ron(
            "Ignore record filters, e.g. `[]`",
            "list[string]",
        )?)
        .inventory_path(prompt_t("Inventory path", "path")?)
        .inventory_commit_force(prompt_yes_or_no(
            "Force on `inventory commit`?",
            "y/N",
        )?)
        .inventory_watch_interval(prompt_t(
            "Interval for `inventory watch`, in milliseconds",
            "number",
        )?);

    // Save
    let default_path =
        default_config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
    let path = prompt_t::<PathBuf>(
        format!("Save location [default: {}]", default_path.display()),
        "path",
    )?
    .map(|p| match p.extension() {
        Some(_) => p,
        None => p.with_extension("toml"),
    })
    .unwrap_or(default_path);
    io::fs::remove_interactive(&path).await?;
    builder.save(path).await?;

    Ok(())
}

#[tracing::instrument(level = "trace")]
async fn show(config: Option<PathBuf>) -> Result<()> {
    let cfg = ConfigOpts::full(config, None)?;
    println!("{}", cfg);
    Ok(())
}
