pub fn exec(cmd: &str) -> String {
    subprocess::Exec::shell(cmd).capture().unwrap().stdout_str()
}
