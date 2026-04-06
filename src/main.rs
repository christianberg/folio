use folio::infrastructure::{Args, Filesystem, Output};

fn main() {
    let code = folio::run(Args::create(), &Filesystem::create(), &Output::create());
    std::process::exit(code);
}
