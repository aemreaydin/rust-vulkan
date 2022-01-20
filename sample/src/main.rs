use vulkan_renderer::instance::VInstance;

fn main() {
    let instance = VInstance::new("Sample", 0).expect("Failed to create instance.");

    println!("Hello, world!");
}
