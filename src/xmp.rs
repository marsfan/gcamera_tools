//! Logic for parsing the XMP data in an image.
#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
use roxmltree::{Document, ExpandedName, Namespace, Node};
use std::fs;

// Namespace consants.
// TODO: Could we use some other structure/enum instead?
// const X_NS: &str = "adobe:ns:meta/";
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const XMP_NOTE_NS: &str = "http://ns.adobe.com/xmp/note/";
const GCAMERA_NS: &str = "http://ns.google.com/photos/1.0/camera/";
const CONTAINER_NS: &str = "http://ns.google.com/photos/1.0/container/";
const ITEM_NS: &str = "http://ns.google.com/photos/1.0/container/item/";

/// Convert an XML attribute in a node to a string.
///
/// # Arguments
/// * `node`: The node to read the attribute from
/// * `namespace`: The namespace of the attribute
/// * `attribute`: The name of the attribute
///
/// # Returns
///  Option holding the attribute converted to a string.
fn attribute_to_str(node: Node, namespace: &str, attribute: &str) -> Option<String> {
    return node
        .attribute((namespace, attribute))
        .map(|n| return String::from(n));
}
/// Convert an XML attribute in a node to a u32.
///
/// # Arguments
/// * `node`: The node to read the attribute from
/// * `namespace`: The namespace of the attribute
/// * `attribute`: The name of the attribute
///
/// # Returns
///  Option holding the attribute converted to a u32.
// FIXME: Better error transformation?
fn attribute_to_u32(
    node: Node,
    namespace: &str,
    attribute: &str,
) -> Result<Option<u32>, ParseIntError> {
    return node
        .attribute((namespace, attribute))
        .map(|n| return n.parse())
        .transpose();
}

/// General information about the XMP data
#[derive(Debug, PartialEq, Eq)]
pub struct Description {
    /// Identifier for extended XMP info in additional JPEG segments.
    extended_xmp_id: Option<String>,

    /// Indicator for the presence of a motion photo.
    motion_photo: Option<u32>,

    /// Indication for teh motion photo version
    motion_photo_version: Option<u32>,

    /// TODO: Figure this out
    motion_photo_timestamp_us: Option<u32>,
}

impl Description {
    /// Create an instance from the XML Element
    /// TODO: Finish Documentation
    pub fn from_xml(xml_element: Node) -> Self {
        return Self {
            extended_xmp_id: attribute_to_str(xml_element, XMP_NOTE_NS, "HasExtendedXMP"),
            motion_photo: attribute_to_u32(xml_element, GCAMERA_NS, "MotionPhoto").unwrap(),
            motion_photo_version: attribute_to_u32(xml_element, GCAMERA_NS, "MotionPhotoVersion")
                .unwrap(),
            motion_photo_timestamp_us: attribute_to_u32(
                xml_element,
                GCAMERA_NS,
                "MotionPhotoPresentationTimestampUs",
            )
            .unwrap(),
        };
    }
}

/// Data about a single resource in the file
#[derive(Debug, PartialEq, Eq)]
pub struct Item {
    /// The mimetype of the resource
    mimetype: String, // TODO: Enum?

    /// The length of the resource
    /// Optional for primary resource.
    length: Option<u32>,

    /// Length in bytes between end of resource and start of next resource
    padding: Option<u32>,

    ///The semantic type of the resource.
    semantic: String, // TODO: Enum?

    /// Optional Parameter to disambiguate items of the same semantic type
    label: Option<String>,

    /// Optional URI string containing relative URI of the item.
    /// Only present if the base media format is ISO/IEC 14496-12
    uri: Option<String>,
}

impl Item {
    /// Create an instance from an XML node
    ///
    ///
    /// TODO: Args
    ///
    /// # Returns
    /// Instance created from an XML Node
    pub fn from_xml(xml_element: Node) -> Self {
        return Self {
            mimetype: attribute_to_str(xml_element, ITEM_NS, "Mime").unwrap(),
            length: attribute_to_u32(xml_element, ITEM_NS, "Length").unwrap(),
            padding: attribute_to_u32(xml_element, ITEM_NS, "Padding").unwrap(),
            semantic: attribute_to_str(xml_element, ITEM_NS, "Semanti").unwrap(),
            label: attribute_to_str(xml_element, ITEM_NS, "Label"),
            uri: attribute_to_str(xml_element, ITEM_NS, "URI"),
        };
    }
}

// TODO: Document
pub struct XMPData {
    pub description: Description,
    pub resources: Vec<Item>,
}

// TODO: Document
impl XMPData {
    pub fn from_xml(document: Document) -> Self {
        let description_node = document
            .descendants()
            .find(|n| {
                return n.tag_name() == ExpandedName::from((RDF_NS, "Description"));
            })
            .unwrap();

        let resource_nodes = document
            .descendants()
            .filter(|n| {
                return n.tag_name() == ExpandedName::from((CONTAINER_NS, "Item"));
            })
            .map(|n| return Item::from_xml(n));

        return Self {
            description: Description::from_xml(description_node),
            resources: resource_nodes.collect(),
        };
    }

    // TODO: Document
    pub fn from_string(xmp_string: String) -> Self {
        return Self::from_xml(Document::parse(&xmp_string).unwrap());
    }
}

pub fn example() {
    let text = String::from_utf8(fs::read("motion.xml").unwrap()).unwrap();
    let doc = Document::parse(&text).unwrap();

    let elems = doc
        .descendants()
        .filter(|n| return n.tag_name().name() == "Item");
    for elem in elems {
        println!(
            "{:?}",
            elem.attribute(("http://ns.google.com/photos/1.0/container/item/", "Mime"))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for the attribute_to_str function
    mod test_attribute_to_str {
        use super::*;

        /// Test the attribute_to_str function when it has a valid value.
        #[test]
        fn test_some() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.example.com", "a"),
                Some(String::from("Hello"))
            );
        }

        /// Test the attribute_to_str function when it does not have a valid value.
        #[test]
        fn test_none() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.example.com", "b"),
                None
            );
        }

        /// Test the attribute_to_str function when namespace is invalid.
        #[test]
        fn test_bad_ns() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.second.example.com", "b"),
                None
            );
        }

        /// Test the attribute_to_str function when there are no namespaces
        #[test]
        fn test_no_ns() {
            let test_xml = "<tagname a=\"Hello\"/>";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.second.example.com", "b"),
                None
            );
        }
    }

    /// Tests for the attribute_to_u32 method
    mod test_attribute_to_u32 {

        use super::*;

        /// Test the attribute_to_u32 function when it has a valid value.
        #[test]
        fn test_some() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_u32(xml_element, "http://ns.example.com", "a"),
                Ok(Some(1))
            );
        }

        /// Test the attribute_to_u32 function when it does not have a valid value.
        #[test]
        fn test_none() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_u32(xml_element, "http://ns.example.com", "b"),
                Ok(None)
            );
        }

        /// Test the attribute_to_u32 function when namespace is invalid.
        #[test]
        fn test_bad_ns() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_u32(xml_element, "http://ns.second.example.com", "b"),
                Ok(None)
            );
        }

        /// Test the attribute_to_u32 function when there are no namespaces
        #[test]
        fn test_no_ns() {
            let test_xml = "<tagname a=\"1\"/>";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_u32(xml_element, "http://ns.example.com", "b"),
                Ok(None)
            );
        }

        /// Test case where the attribute cannot be parsed to a string.
        #[test]
        #[should_panic]
        fn test_not_parseable() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();
            attribute_to_u32(xml_element, "http://ns.example.com", "a").unwrap();
        }
    }

    /// Tests of the Description struct
    mod test_description {

        use super::*;

        /// Basic initialization test.
        #[test]
        fn test_init() {
            let test_xml =
                "<rdf:Description xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\"
                    rdf:about=\"\"
                    xmlns:xmpNote=\"http://ns.adobe.com/xmp/note/\"
                    xmlns:GCamera=\"http://ns.google.com/photos/1.0/camera/\"
                    xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"
                    xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"
                    xmpNote:HasExtendedXMP=\"DD558CA2166AEC119A42CDFB02D4F1EF\"
                    GCamera:MotionPhoto=\"1\"
                    GCamera:MotionPhotoVersion=\"1\"
                    GCamera:MotionPhotoPresentationTimestampUs=\"968644\"/>";

            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| n.tag_name().name() == "Description")
                .unwrap();
            let description = Description::from_xml(xml_element);

            assert_eq!(
                description,
                Description {
                    extended_xmp_id: Some(String::from("DD558CA2166AEC119A42CDFB02D4F1EF")),
                    motion_photo: Some(1),
                    motion_photo_version: Some(1),
                    motion_photo_timestamp_us: Some(968644),
                },
            )
        }
    }

    mod test_item {
        use super::*;
        /// Basic initialization tests.
        #[test]
        fn test_init() {
            let test_xml = "<Container:Item
              Item:Mime=\"video/mp4\"
              Item:Semantic=\"MotionPhoto\"
              Item:Length=\"4906025\"
              Item:Padding=\"0\"
              xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"
              xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"/>";

            let document = Document::parse(&test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| n.tag_name().name() == "Description")
                .unwrap();
            let item = Item::from_xml(xml_element);

            assert_eq!(
                item,
                Item {
                    mimetype: String::from("video/mp4"),
                    length: Some(4906025),
                    padding: Some(0),
                    semantic: String::from("MotionPhoto"),
                    label: None,
                    uri: None
                }
            )
        }
    }
}
