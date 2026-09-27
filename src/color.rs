use std::{fmt::Display, ops::Deref};

pub struct Colored {
    pub text: String,
}

impl<'a> From<&&'a str> for Colored {
    fn from(s: &&'a str) -> Self {
        Self {
            text: String::from(*s),
        }
    }
}

impl From<&String> for Colored {
    fn from(s: &String) -> Self {
        Self { text: s.clone() }
    }
}

impl Display for Colored {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)?;
        Ok(())
    }
}

impl Deref for Colored {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

#[allow(dead_code)]
pub trait Color {
    fn black(&self) -> Colored;
    fn red(&self) -> Colored;
    fn green(&self) -> Colored;
    fn yellow(&self) -> Colored;
    fn blue(&self) -> Colored;
    fn magenta(&self) -> Colored;
    fn cyan(&self) -> Colored;
    fn white(&self) -> Colored;

    fn bright_black(&self) -> Colored;
    fn bright_red(&self) -> Colored;
    fn bright_green(&self) -> Colored;
    fn bright_yellow(&self) -> Colored;
    fn bright_blue(&self) -> Colored;
    fn bright_magenta(&self) -> Colored;
    fn bright_cyan(&self) -> Colored;
    fn bright_white(&self) -> Colored;

    fn bold(&self) -> Colored;
    fn dimmed(&self) -> Colored;
    fn italic(&self) -> Colored;
    fn underline(&self) -> Colored;
    fn strike(&self) -> Colored;
}

impl Color for Colored {
    fn black(&self) -> Colored {
        Colored {
            text: format!("\x1b[30m{}\x1b[0m", self.text),
        }
    }

    fn red(&self) -> Colored {
        Colored {
            text: format!("\x1b[31m{}\x1b[0m", self.text),
        }
    }

    fn green(&self) -> Colored {
        Colored {
            text: format!("\x1b[32m{}\x1b[0m", self.text),
        }
    }

    fn yellow(&self) -> Colored {
        Colored {
            text: format!("\x1b[33m{}\x1b[0m", self.text),
        }
    }

    fn blue(&self) -> Colored {
        Colored {
            text: format!("\x1b[34m{}\x1b[0m", self.text),
        }
    }

    fn magenta(&self) -> Colored {
        Colored {
            text: format!("\x1b[35m{}\x1b[0m", self.text),
        }
    }

    fn cyan(&self) -> Colored {
        Colored {
            text: format!("\x1b[36m{}\x1b[0m", self.text),
        }
    }

    fn white(&self) -> Colored {
        Colored {
            text: format!("\x1b[37m{}\x1b[0m", self.text),
        }
    }

    fn bright_black(&self) -> Colored {
        Colored {
            text: format!("\x1b[30;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_red(&self) -> Colored {
        Colored {
            text: format!("\x1b[31;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_green(&self) -> Colored {
        Colored {
            text: format!("\x1b[32;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_yellow(&self) -> Colored {
        Colored {
            text: format!("\x1b[33;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_blue(&self) -> Colored {
        Colored {
            text: format!("\x1b[34;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_magenta(&self) -> Colored {
        Colored {
            text: format!("\x1b[35;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_cyan(&self) -> Colored {
        Colored {
            text: format!("\x1b[36;1m{}\x1b[0m", self.text),
        }
    }

    fn bright_white(&self) -> Colored {
        Colored {
            text: format!("\x1b[37;1m{}\x1b[0m", self.text),
        }
    }

    fn bold(&self) -> Colored {
        Colored {
            text: format!("\x1b[1m{}\x1b[0m", self.text),
        }
    }
    fn dimmed(&self) -> Colored {
        Colored {
            text: format!("\x1b[2m{}\x1b[0m", self.text),
        }
    }
    fn italic(&self) -> Colored {
        Colored {
            text: format!("\x1b[3m{}\x1b[0m", self.text),
        }
    }
    fn underline(&self) -> Colored {
        Colored {
            text: format!("\x1b[4m{}\x1b[0m", self.text),
        }
    }
    fn strike(&self) -> Colored {
        Colored {
            text: format!("\x1b[9m{}\x1b[0m", self.text),
        }
    }
}

impl Color for &str {
    fn black(&self) -> Colored {
        Colored::from(self).black()
    }

    fn red(&self) -> Colored {
        Colored::from(self).red()
    }

    fn green(&self) -> Colored {
        Colored::from(self).green()
    }

    fn yellow(&self) -> Colored {
        Colored::from(self).yellow()
    }

    fn blue(&self) -> Colored {
        Colored::from(self).blue()
    }

    fn magenta(&self) -> Colored {
        Colored::from(self).magenta()
    }

    fn cyan(&self) -> Colored {
        Colored::from(self).cyan()
    }

    fn white(&self) -> Colored {
        Colored::from(self).white()
    }

    fn bright_black(&self) -> Colored {
        Colored::from(self).bright_black()
    }

    fn bright_red(&self) -> Colored {
        Colored::from(self).bright_red()
    }

    fn bright_green(&self) -> Colored {
        Colored::from(self).bright_green()
    }

    fn bright_yellow(&self) -> Colored {
        Colored::from(self).bright_yellow()
    }

    fn bright_blue(&self) -> Colored {
        Colored::from(self).bright_blue()
    }

    fn bright_magenta(&self) -> Colored {
        Colored::from(self).bright_magenta()
    }

    fn bright_cyan(&self) -> Colored {
        Colored::from(self).bright_cyan()
    }

    fn bright_white(&self) -> Colored {
        Colored::from(self).bright_white()
    }

    fn bold(&self) -> Colored {
        Colored::from(self).bold()
    }

    fn dimmed(&self) -> Colored {
        Colored::from(self).dimmed()
    }

    fn italic(&self) -> Colored {
        Colored::from(self).italic()
    }

    fn underline(&self) -> Colored {
        Colored::from(self).underline()
    }

    fn strike(&self) -> Colored {
        Colored::from(self).strike()
    }
}

impl Color for String {
    fn black(&self) -> Colored {
        Colored::from(self).black()
    }

    fn red(&self) -> Colored {
        Colored::from(self).red()
    }

    fn green(&self) -> Colored {
        Colored::from(self).green()
    }

    fn yellow(&self) -> Colored {
        Colored::from(self).yellow()
    }

    fn blue(&self) -> Colored {
        Colored::from(self).blue()
    }

    fn magenta(&self) -> Colored {
        Colored::from(self).magenta()
    }

    fn cyan(&self) -> Colored {
        Colored::from(self).cyan()
    }

    fn white(&self) -> Colored {
        Colored::from(self).white()
    }

    fn bright_black(&self) -> Colored {
        Colored::from(self).bright_black()
    }

    fn bright_red(&self) -> Colored {
        Colored::from(self).bright_red()
    }

    fn bright_green(&self) -> Colored {
        Colored::from(self).bright_green()
    }

    fn bright_yellow(&self) -> Colored {
        Colored::from(self).bright_yellow()
    }

    fn bright_blue(&self) -> Colored {
        Colored::from(self).bright_blue()
    }

    fn bright_magenta(&self) -> Colored {
        Colored::from(self).bright_magenta()
    }

    fn bright_cyan(&self) -> Colored {
        Colored::from(self).bright_cyan()
    }

    fn bright_white(&self) -> Colored {
        Colored::from(self).bright_white()
    }

    fn bold(&self) -> Colored {
        Colored::from(self).bold()
    }

    fn dimmed(&self) -> Colored {
        Colored::from(self).dimmed()
    }

    fn italic(&self) -> Colored {
        Colored::from(self).italic()
    }

    fn underline(&self) -> Colored {
        Colored::from(self).underline()
    }

    fn strike(&self) -> Colored {
        Colored::from(self).strike()
    }
}

#[test]
fn test() {
    println!("{}", "red".red());
    println!("{}", "blud_bold".blue().bold());
    println!("{}", "green_italic".green().italic());
    println!("{}", "white_bold_italic".white().bold().italic());
    println!("{}", "magenta_underline".magenta().underline());
    println!("{}", "italic_red".italic().red());
    println!("{}", "underline_blue".underline().blue());
    println!("{}", "strike".strike().blue());
}
