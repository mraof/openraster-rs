use serde::{
    Deserialize, Serialize,
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename = "image")]
pub struct LayerStack {
    #[serde(rename = "@version")]
    pub version: String,
    #[serde(rename = "@w")]
    pub w: u32,
    #[serde(rename = "@h")]
    pub h: u32,
    #[serde(rename = "@xres")]
    pub xres: Option<u32>,
    #[serde(rename = "@yres")]
    pub yres: Option<u32>,
    pub stack: Stack,
}

impl LayerStack {
    pub fn xml(&self) -> String {
        serde_xml_rs::to_string(&self).unwrap()
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self {
            version: "0.0.6".to_string(),
            w: Default::default(),
            h: Default::default(),
            xres: Default::default(),
            yres: Default::default(),
            stack: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
pub enum Visibility {
    #[serde(rename = "visible")]
    #[default]
    Visible,
    #[serde(rename = "hidden")]
    Hidden,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
pub enum Isolation {
    #[serde(rename = "isolate")]
    #[default]
    Isolate,
    #[serde(rename = "auto")]
    Auto,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
pub enum CompositeOp {
    #[serde(rename = "svg:src-over")]
    #[default]
    SrcOver,
    #[serde(rename = "svg:multiply")]
    Multiply,
    #[serde(rename = "svg:screen")]
    Screen,
    #[serde(rename = "svg:overlay")]
    Overlay,
    #[serde(rename = "svg:darken")]
    Darken,
    #[serde(rename = "svg:lighten")]
    Lighten,
    #[serde(rename = "svg:color-dodge")]
    ColorDodge,
    #[serde(rename = "svg:color-burn")]
    ColorBurn,
    #[serde(rename = "svg:hard-light")]
    HardLight,
    #[serde(rename = "svg:soft-light")]
    SoftLight,
    #[serde(rename = "svg:difference")]
    Difference,
    #[serde(rename = "svg:color")]
    Color,
    #[serde(rename = "svg:luminosity")]
    Luminosity,
    #[serde(rename = "svg:hue")]
    Hue,
    #[serde(rename = "svg:saturation")]
    Saturation,
    #[serde(rename = "svg:plus")]
    Lighter,
    #[serde(rename = "svg:dst-in")]
    DestinationIn,
    #[serde(rename = "svg:dst-out")]
    DestinationOut,
    #[serde(rename = "svg:src-atop")]
    SourceAtop,
    #[serde(rename = "svg:dst-atop")]
    DestinationAtop,
}

#[derive(Clone, Default, Debug, Deserialize, Serialize, PartialEq)]
pub struct Layer {
    #[serde(rename = "@src")]
    pub src: String,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@x")]
    pub x: Option<i32>,
    #[serde(rename = "@y")]
    pub y: Option<i32>,
    #[serde(rename = "@opacity")]
    pub opacity: Option<f64>,
    #[serde(rename = "@visibility")]
    pub visibility: Option<Visibility>,
    #[serde(rename = "@composite-op")]
    pub composite_op: Option<CompositeOp>,
}

impl Layer {
    pub fn visible(&self) -> bool {
        self.visibility
            .as_ref()
            .is_none_or(|v| *v == Visibility::Visible)
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Stack {
    pub name: Option<String>,
    pub opacity: Option<f64>,
    pub visibility: Option<Visibility>,
    pub composite_op: Option<CompositeOp>,
    pub isolation: Option<Isolation>,
    /// Vec for both <layer> and <stack> to preserve order
    pub sub: Vec<Sub>,
}

impl Stack {
    pub fn visible(&self) -> bool {
        self.visibility
            .as_ref()
            .is_none_or(|v| *v == Visibility::Visible)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Sub {
    #[serde(rename = "layer")]
    Layer(Layer),
    #[serde(rename = "stack")]
    Stack(Stack),
}

impl Sub {
    pub fn visible(&self) -> bool {
        match self {
            Sub::Layer(layer) => layer.visible(),
            Sub::Stack(stack) => stack.visible(),
        }
    }
}

// Tried figuring out a way to handle deserializing layers and stacks in a way that'd preserve order with serde derive but couldn't
impl<'de> Deserialize<'de> for Stack {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "@name",
            "@opacity",
            "@visibility",
            "@composite-op",
            "@isolation",
            "stack",
            "layer",
        ];
        struct StackVisitor;

        impl<'de> Visitor<'de> for StackVisitor {
            type Value = Stack;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Stack")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Stack, V::Error>
            where
                V: MapAccess<'de>,
            {
                //#[serde(rename = "@name")]
                let mut name: Option<String> = None;
                //#[serde(rename = "@opacity")]
                let mut opacity: Option<f64> = None;
                //#[serde(rename = "@visibility")]
                let mut visibility: Option<Visibility> = None;
                //#[serde(rename = "@composite-op")]
                let mut composite_op: Option<CompositeOp> = None;
                //#[serde(rename = "@isolation")]
                let mut isolation: Option<Isolation> = None;
                let mut sub: Vec<Sub> = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "@name" => {
                            if name.is_some() {
                                return Err(serde::de::Error::duplicate_field("@name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        "@opacity" => {
                            if opacity.is_some() {
                                return Err(serde::de::Error::duplicate_field("@opacity"));
                            }
                            opacity = Some(map.next_value()?);
                        }
                        "@visibility" => {
                            if visibility.is_some() {
                                return Err(serde::de::Error::duplicate_field("@visibility"));
                            }
                            visibility = Some(map.next_value()?);
                        }
                        "@composite-op" => {
                            if composite_op.is_some() {
                                return Err(serde::de::Error::duplicate_field("@composite-op"));
                            }
                            composite_op = Some(map.next_value()?);
                        }
                        "@isolation" => {
                            if isolation.is_some() {
                                return Err(serde::de::Error::duplicate_field("@isolation"));
                            }
                            isolation = Some(map.next_value()?);
                        }
                        "stack" => {
                            sub.push(Sub::Stack(map.next_value()?));
                        }
                        "layer" => {
                            sub.push(Sub::Layer(map.next_value()?));
                        }
                        field => return Err(serde::de::Error::unknown_field(field, FIELDS)),
                    }
                }
                Ok(Stack {
                    name,
                    opacity,
                    visibility,
                    composite_op,
                    isolation,
                    sub,
                })
            }
        }
        deserializer.deserialize_struct("Stack", FIELDS, StackVisitor)
    }
}

impl Serialize for Stack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Stack", 6)?;
        state.serialize_field("@name", &self.name)?;
        state.serialize_field("@opacity", &self.opacity)?;
        state.serialize_field("@visibility", &self.visibility)?;
        state.serialize_field("@composite-op", &self.composite_op)?;
        state.serialize_field("@isolation", &self.isolation)?;
        for s in &self.sub {
            match s {
                Sub::Layer(layer) => state.serialize_field("layer", layer)?,
                Sub::Stack(stack) => state.serialize_field("stack", stack)?,
            }
        }
        state.end()
    }
}
