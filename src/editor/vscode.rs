use super::Editor;

pub struct VSCodeEditor;

impl Editor for VSCodeEditor {
    fn name(&self) -> &str {
        "VSCode"
    }

    fn command(&self) -> &str {
        "code"
    }
}
