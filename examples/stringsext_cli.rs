use std::path::PathBuf;

pub fn main() -> stringexts::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        println!("please give filename to extract strings!");
        return Ok(());
    }
    let file = args[1].to_owned();

    let mut str_scan =
        stringexts::StringsScanner::new(None, &[], None, false, None, None, None, None)?;
    let res = str_scan.run(vec![PathBuf::from(file)])?;
    println!("{:?}", res);
    Ok(())
}
