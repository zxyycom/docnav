fn main() {
    let exit = docnav::run(
        std::env::args().skip(1),
        std::io::stdin(),
        std::io::stdout(),
        std::io::stderr(),
    );
    std::process::exit(exit);
}
