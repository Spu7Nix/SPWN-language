use spwn::Compiler;
fn main() {
    let included_paths = vec![
                std::env::current_dir().expect("Cannot access current directory"),
                std::env::current_exe().expect("Cannot access directory of executable").parent().expect("Executable must be in some directory").to_path_buf(),
    ];
	let output = match Compiler::_run("
		let c1 = counter(2i)
		wait(2)
		c1 += 10
		".to_string(), included_paths, true){
		Ok(s) => s,
		Err(e) => panic!("We did an oopsie, message is {:?}", e)
	};
    println!("Hello, world! This is the level string {}", output);
}
