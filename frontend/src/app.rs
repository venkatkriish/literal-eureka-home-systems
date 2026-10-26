use crate::api::client::{sample_bills, sample_users, Client};

pub fn render_app() -> String {
    let client = Client::new("http://localhost:3000");
    let dashboard = client.dashboard_from_seed(sample_users(), sample_bills());
    format!(
        "<section class='app'><h1>Electricity Billing</h1><p>Total users: {}</p><p>Total bills: {}</p><p>Revenue: ₹{}</p></section>",
        dashboard.summary.total_users,
        dashboard.summary.total_bills,
        dashboard.summary.total_revenue
    )
}
