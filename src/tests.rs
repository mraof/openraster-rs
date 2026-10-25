use svg::Document;

use crate::stack::{Layer, LayerStack, Stack, Sub};

use super::*;

#[test]
fn create_ora() {
    let mut layer1 = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut layer1, 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let data = [128; 8];
        writer.write_image_data(&data).unwrap();
    }
    let mut layer2 = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut layer2, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        let data = [64, 0, 0, 64];
        writer.write_image_data(&data).unwrap();
    }
    let mut bg = Vec::new();
    {
        let rect = svg::node::element::Rectangle::new()
            .set("fill", "white")
            .set("width", 2)
            .set("height", 2);
        let document = Document::new()
            .set("viewBox", (0, 0, 2, 2))
            .add(rect);
        svg::write(&mut bg, &document).unwrap();
    }
    //let mut bg = Vec::new();
    let mut data = HashMap::new();
    data.insert("data/layer1.png".to_string(), layer1);
    data.insert("data/layer2.png".to_string(), layer2);
    data.insert("data/layer3.svg".to_string(), bg);
    let stack = Stack {
        sub: vec![
            Sub::Layer(Layer {
                src: "data/layer1.png".to_string(),
                ..Layer::default()
            }),
            Sub::Layer(Layer {
                src: "data/layer2.png".to_string(),
                name: Some("Red ☺".to_string()),
                x: Some(1),
                ..Layer::default()
            }),
            Sub::Layer(Layer {
                src: "data/layer3.svg".to_string(),
                name: Some("Background".to_string()),
                ..Layer::default()
            }),
        ],
        ..Stack::default()
    };
    let mut ora = OpenRaster::new(2, 2);
    ora.stack.stack = stack;
    ora.data = data;
    ora.render();
    ora.save("test.ora").unwrap();
}

#[test]
fn serialize_stack() {
    let mut s = stack::LayerStack::default();
    s.stack.sub.push(stack::Sub::Layer(Layer::default()));
    s.stack.sub.push(stack::Sub::Layer(Layer::default()));
    s.stack.sub.push(stack::Sub::Stack(Stack::default()));
    s.stack.sub.push(stack::Sub::Layer(Layer::default()));
    println!("{}", serde_xml_rs::to_string(&s).unwrap());
}

#[test]
fn parse_example_stack() {
    let correct = LayerStack {
        version: "0.0.3".to_string(),
        w: 300,
        h: 177,
        xres: Some(600),
        yres: Some(600),
        stack: Stack {
            sub: vec![
                Sub::Stack(Stack {
                    sub: vec![Sub::Layer(Layer {
                        name: Some("OpenRaster Logo".to_string()),
                        src: "data/hw.svg".to_string(),
                        x: Some(15),
                        y: Some(5),
                        ..Layer::default()
                    })],
                    ..Stack::default()
                }),
                Sub::Layer(Layer {
                    name: Some("layer1".to_string()),
                    src: "data/image1.png".to_string(),
                    ..Layer::default()
                }),
            ],
            ..Stack::default()
        },
    };
    let example = r#"<?xml version='1.0' encoding='UTF-8'?>
<image version="0.0.3" w="300" h="177" xres="600" yres="600">
  <stack>
    <stack>
        <layer name="OpenRaster Logo" src="data/hw.svg" x="15" y="5" />
    </stack>
    <!-- filters are syntactically permissible, but not valid for baseline -->
    <layer name="layer1" src="data/image1.png" />
  </stack>
</image>"#;
    let image: LayerStack = serde_xml_rs::from_str(example).unwrap();
    assert_eq!(image, correct)
}
