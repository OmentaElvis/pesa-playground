use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use tokio::{
    net::TcpStream,
    time::{Instant, timeout},
};

use crate::{
    AppContext,
    accounts::user_profiles,
    business, projects,
    self_test::{callback::CallbackManager, context::TestContext, runner::TestStep},
};

pub struct InitProjectTest;

async fn port_is_open(addr: SocketAddr) -> bool {
    timeout(Duration::from_secs(1), TcpStream::connect(addr))
        .await
        .is_ok()
}

impl TestStep for InitProjectTest {
    async fn run(
        &self,
        context: &mut TestContext,
        _callback_manager: &mut CallbackManager,
    ) -> anyhow::Result<()> {
        let app: &AppContext = &context.app_context;
        context.log("Creating test business...").await;

        let initial_working_balance = 100_000.0;
        let initial_utility_balance = 5000.0;
        let shortcode = "9999991".to_string();
        let name = "Test Framework biz".to_string();

        // ==== Business =====
        let business = business::ui::create_business(
            app,
            business::CreateBusiness {
                name: name.clone(),
                short_code: shortcode.clone(),
                initial_working_balance,
                initial_utility_balance,
            },
        )
        .await
        .context("Failed to create a new business with shortcode: 9999991")?;

        let business = business::ui::get_business(app, business.id)
            .await
            .context("Failed to fetch business from db")
            .expect("Business not found. Expected to be created but missing");

        context.log(&format!("{:#?}", business)).await;

        assert_eq!(
            business.mmf_account.balance,
            (initial_working_balance * 100.0) as i64,
            "Expected funds to be deposited in mmf working account."
        );

        assert_eq!(
            business.utility_account.balance,
            (initial_utility_balance * 100.0) as i64,
            "Expected funds to be deposited in utility working account."
        );
        context.log("Creating test project...").await;

        // ==== Project =====
        let project = projects::ui::create_project(
            app,
            projects::CreateProject {
                business_id: business.id,
                name: "Test Framework Biz Project".to_string(),
                callback_url: None,
                simulation_mode: projects::SimulationMode::Realistic,
                stk_delay: 0,
                prefix: None,
            },
        )
        .await
        .context("Failed to create project")?;
        context.log(&format!("{:#?}", project)).await;

        // ==== Users =====
        context.log("Generating test users...").await;
        let mut users = user_profiles::ui::generate_users(3)
            .await
            .context("Failed to generate users.")?;

        assert_eq!(users.len(), 3);
        let broke_user = users.pop().unwrap();
        let rich_user = users.pop().unwrap();
        let average_user = users.pop().unwrap();

        let broke_user = user_profiles::ui::create_user(
            app,
            broke_user.name,
            broke_user.phone,
            0.0,
            broke_user.pin,
        )
        .await
        .context("Failed to create poor test user")?;
        context.log(&format!("Poor {:#?}", broke_user)).await;

        let rich_user = user_profiles::ui::create_user(
            app,
            rich_user.name,
            rich_user.phone,
            100_000.0,
            rich_user.pin,
        )
        .await
        .context("Failed to create rich test user")?;
        context.log(&format!("Rich {:#?}", rich_user)).await;

        let average_user = user_profiles::ui::create_user(
            app,
            average_user.name,
            average_user.phone,
            5000.0,
            average_user.pin,
        )
        .await
        .context("Failed to create average test user")?;
        context.log(&format!("Average {:#?}", average_user)).await;

        context.log("Starting project sandbox.").await;

        // === Start project sandbox ===
        let start = Instant::now();
        let url =
            crate::sandboxes::ui::start_sandbox(app, project.id, Some("127.0.0.1".to_string()))
                .await
                .context("Failed to start project sandbox")?;

        let addr = url
            .strip_prefix("http://")
            .expect("Expected url to contain http:// prefix")
            .parse::<SocketAddr>()
            .context(format!(
                "Failed to parse sandbox address ({}) to SocketAddr",
                url
            ))?;

        assert!(port_is_open(addr).await, "Expected sandbox to be running");
        let startup_time = start.elapsed();
        context
            .log(&format!(
                "Sandbox started in {} µs",
                startup_time.as_micros()
            ))
            .await;

        context.log(&format!("URL: {}", url)).await;

        // save these for the next tests
        context.set("project", &project)?;
        context.set("business", &business)?;
        context.set("broke_user", &broke_user)?;
        context.set("rich_user", &rich_user)?;
        context.set("average_user", &average_user)?;
        context.set("base_url", &url)?;

        Ok(())
    }
}
