//! Simple Slint test to verify basic functionality

slint::slint! {
    export component TestWindow inherits Window {
        title: "Simple Test";
        preferred-width: 400px;
        preferred-height: 300px;
        
        Text {
            text: "Hello, Slint!";
            font-size: 24px;
            horizontal-alignment: center;
            vertical-alignment: center;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating simple Slint window...");
    
    let window = TestWindow::new()?;
    println!("Window created successfully!");
    
    println!("Running window...");
    window.run()?;
    
    println!("Window closed.");
    Ok(())
}
