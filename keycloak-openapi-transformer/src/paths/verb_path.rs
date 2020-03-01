use std::str::FromStr;

pub struct VerbPath {
    pub verb: String,
    path: String,
}

impl FromStr for VerbPath {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split_whitespace();
        let verb = split.next().ok_or(())?.to_string();
        let path = split.next().ok_or(())?.to_string();

        Ok(Self { verb, path })
    }
}

impl VerbPath {
    pub fn unrepresentable(&self) -> bool {
        self.path == "/{any}"
    }

    pub fn repeating_ids(&self) -> Option<usize> {
        let c = self.path.matches("{id}").count();
        if c > 1 {
            Some(c)
        } else {
            None
        }
    }

    pub fn path(&self) -> String {
        match self.repeating_ids() {
            None => self.path.to_string(),
            Some(_) => {
                let mut sections = self.path.split("{id}");
                let mut path = sections.next().unwrap().to_string();
                for (section, i) in sections.zip(1..) {
                    path.push_str(&format!("{{id{}}}", i));
                    path.push_str(section);
                }
                path
            }
        }
    }
}
