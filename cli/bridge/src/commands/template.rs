use anyhow::Result;
use console::style;

pub async fn handle(list: bool) -> Result<()> {
    if list {
        println!("{}", style("Available Templates:").bold().cyan());
        println!(
            "  - {} (default): Basic structure with Python & Node.js setup",
            style("basic").green()
        );
        // Future templates
        println!(
            "  - {} (coming soon): Library structure",
            style("library").dim()
        );
        println!(
            "  - {} (coming soon): WebSocket server",
            style("websocket").dim()
        );
        println!(
            "  - {} (coming soon): DataFrame processing",
            style("dataframe").dim()
        );
        println!(
            "  - {} (coming soon): Only Python Setup",
            style("onlypython").dim()
        );
        println!(
            "  - {} (coming soon): Only Node.js Setup",
            style("onlynode").dim()
        );
    } else {
        println!(
            "Use {} to see available templates.",
            style("bridge template --list").yellow()
        );
    }

    Ok(())
}
