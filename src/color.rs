pub(crate) trait Color {
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
    fn cyan(&self) -> String;
}

impl Color for str {
    fn red(&self) -> String {
        format!("\x1b[31m{self}\x1b[0m")
    }

    fn green(&self) -> String {
        format!("\x1b[32m{self}\x1b[0m")
    }

    fn yellow(&self) -> String {
        format!("\x1b[33m{self}\x1b[0m")
    }

    fn blue(&self) -> String {
        format!("\x1b[34m{self}\x1b[0m")
    }

    fn cyan(&self) -> String {
        format!("\x1b[36m{self}\x1b[0m")
    }
}
