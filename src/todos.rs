use mlua::{Lua, LuaSerdeExt};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

#[derive(Serialize, Deserialize, Debug)]
pub struct Todo {
    pub text: String,
    pub checked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedTodos {
    #[serde(skip)]
    pub path: String,
    pub todos: Vec<Todo>,
}

impl SavedTodos {
    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        let lua_script = std::fs::read_to_string(path)?;
        let lua = Lua::new();

        let lua_value = lua.load(&lua_script).eval().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        let mut saved_todos: SavedTodos = lua.from_value(lua_value).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;
        saved_todos.path = path.to_string();
        Ok(saved_todos)
    }

    pub fn add_todo(&mut self, text: String) {
        self.todos.push(Todo {
            text,
            checked: false,
        });
    }

    pub fn check(&mut self, index: usize) -> Result<(), std::io::Error> {
        let todo = self.todos.get_mut(index).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid index: {}", index),
            )
        })?;
        todo.checked = !todo.checked;
        Ok(())
    }

    pub fn remove_todo(&mut self, index: usize) -> Result<(), std::io::Error> {
        if index >= self.todos.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid index: {}", index),
            ));
        }
        self.todos.remove(index);
        Ok(())
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let mut lua_code = String::new();
        writeln!(lua_code, "return {{").unwrap();
        writeln!(lua_code, "    [\"todos\"] = {{").unwrap();

        for (i, todo) in self.todos.iter().enumerate() {
            let escaped_text = escape(&todo.text);
            writeln!(
                lua_code,
                "        [{}] = {{ [\"checked\"] = {}, [\"text\"] = \"{}\" }},",
                i + 1,
                todo.checked,
                escaped_text
            )
            .unwrap();
        }

        writeln!(lua_code, "    }},").unwrap();
        writeln!(lua_code, "}}").unwrap();

        std::fs::write(&self.path, lua_code)
    }
}

// Helper function to escape special characters in Lua strings
fn escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.escape_default())
        .filter(|c| *c != '\\' || c == &'\\')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        fs::write(&file, content).unwrap();
        file
    }

    #[test]
    fn test_full_lifecycle() {
        let lua_content = r#"return {
            todos = {
                [1] = { checked = false, text = "Initial item" },
            }
        }"#;
        
        let test_file = create_test_file(lua_content);
        let path = test_file.path().to_str().unwrap();

        let mut saved = SavedTodos::load(path).unwrap();
        assert_eq!(saved.todos.len(), 1);
        assert_eq!(saved.todos[0].text, "Initial item");

        saved.add_todo("New item".into());
        assert_eq!(saved.todos.len(), 2);
        assert!(!saved.todos[1].checked);

        saved.check(1).unwrap();
        assert!(saved.todos[1].checked);

        saved.remove_todo(0).unwrap();
        assert_eq!(saved.todos.len(), 1);

        saved.save().unwrap();

        let reloaded = SavedTodos::load(path).unwrap();
        assert_eq!(reloaded.todos.len(), 1);
        assert_eq!(reloaded.todos[0].text, "New item");
        assert!(reloaded.todos[0].checked);
    }

    #[test]
    fn test_invalid_operations() {
        let mut saved = SavedTodos {
            path: "".into(),
            todos: vec![Todo {
                text: "test".into(),
                checked: false,
            }],
        };
        assert!(saved.check(99).is_err());
        assert!(saved.remove_todo(99).is_err());
    }

    #[test]
    fn test_special_characters() {
        let test_file = create_test_file(r#"return { todos = {} }"#);
        let path = test_file.path().to_str().unwrap();

        let mut saved = SavedTodos::load(path).unwrap();
        saved.add_todo(r#"Quote " and backslash \"#.into());
        saved.save().unwrap();

        let reloaded = SavedTodos::load(path).unwrap();
        assert_eq!(reloaded.todos[0].text, r#"Quote " and backslash \"#);
    }

    #[test]
    fn test_data_dir() {
        let user = env!("USER");
        let path = format!("/home/{user}/.config/koreader/settings/todos.lua");
        
        if std::path::Path::new(&path).exists() {
            SavedTodos::load(&path).unwrap();
        }
    }
}
