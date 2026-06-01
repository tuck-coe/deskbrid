pub(crate) fn parse_u64_arg(args: &[String], name: &str) -> Option<u64> {
    let pos = args.iter().position(|a| a == name)?;
    args.get(pos + 1).and_then(|v| v.parse::<u64>().ok())
}

pub(crate) fn ok_json(msg: Option<&str>) {
    match msg {
        Some(m) => println!("{{\"ok\": true, \"note\": \"{}\"}}", m),
        None => println!("{{\"ok\": true}}"),
    }
}

#[allow(dead_code)]
pub(crate) fn err_json(msg: &str) {
    println!("{{\"ok\": false, \"error\": \"{}\"}}", msg);
}
