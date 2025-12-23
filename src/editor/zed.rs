use super::Editor;

pub struct ZedEditor;

impl Editor for ZedEditor {
    fn name(&self) -> &str {
        "Zed"
    }

    fn command(&self) -> &str {
        "zed"
    }
}
