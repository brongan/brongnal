#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051";
    println!("Mock server starting on {}", addr);
    
    let listener = hub::mock_server::bind(addr).await?;
    
    println!("Listening... Press Ctrl+C to stop.");
    hub::mock_server::serve(listener, async {
        let _ = tokio::signal::ctrl_c().await;
        println!("Shutting down...");
    }).await?;
    
    Ok(())
}
