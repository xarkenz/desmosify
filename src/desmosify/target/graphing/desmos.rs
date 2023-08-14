use json::JsonValue;

pub mod colors {
    #[derive(Copy, Clone, Debug)]
    pub struct Color {
        r: u8,
        g: u8,
        b: u8,
    }

    impl Color {
        pub const fn new(r: u8, g: u8, b: u8) -> Self {
            Self { r, g, b }
        }
    }

    impl ToString for Color {
        fn to_string(&self) -> String {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }

    pub const RED: Color = Color::new(0xC7, 0x44, 0x40);
    pub const BLUE: Color = Color::new(0x2D, 0x70, 0xB3);
    pub const GREEN: Color = Color::new(0x38, 0x8C, 0x46);
    pub const PURPLE: Color = Color::new(0x60, 0x42, 0xA6);
    pub const ORANGE: Color = Color::new(0xFA, 0x7E, 0x19);
    pub const BLACK: Color = Color::new(0x00, 0x00, 0x00);
}

pub trait Item {
    fn type_name(&self) -> &str;
    fn id(&self) -> &str;
    fn folder_id(&self) -> Option<&str>;
    fn to_json(&self) -> JsonValue;
}

pub struct FolderItem {
    id: String,
    title: String,
    collapsed: bool,
}

impl Item for FolderItem {
    fn type_name(&self) -> &str {
        "folder"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn folder_id(&self) -> Option<&str> {
        None
    }

    fn to_json(&self) -> JsonValue {
        json::object!{
            "type": self.type_name(),
            "id": self.id(),
            "title": self.title,
            "collapsed": self.collapsed,
        }
    }
}

pub struct ExpressionRootItem {
    id: String,
    folder_id: Option<String>,
    content: (), // TODO
}

impl Item for ExpressionRootItem {
    fn type_name(&self) -> &str {
        "expression"
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn folder_id(&self) -> Option<&str> {
        self.folder_id.map(String::as_str)
    }

    fn to_json(&self) -> JsonValue {
        todo!()
    }
}

pub struct Ticker {
    handler: (), // TODO
}

pub struct Expressions {
    list: Vec<Box<dyn Item>>,
    ticker: Option<Ticker>,
}