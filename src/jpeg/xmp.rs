/*
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https: //mozilla.org/MPL/2.0/.
*/
//! Logic for parsing the XMP data in an image.
use roxmltree::{Document, ExpandedName, Node};
use std::str;

use crate::errors::GCameraError;
use crate::jpeg::marker::JpegMarker;

use crate::jpeg::jpeg_components::JpegSegment;

// Namespace consants.

/// RDF Namespace
const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
/// XMP Note Namespace
const XMP_NOTE_NS: &str = "http://ns.adobe.com/xmp/note/";

/// Google generic camera info.
const GCAMERA_NS: &str = "http://ns.google.com/photos/1.0/camera/";

/// Google resource container info
const CONTAINER_NS: &str = "http://ns.google.com/photos/1.0/container/";

/// Google Resource item info
const ITEM_NS: &str = "http://ns.google.com/photos/1.0/container/item/";

/// String that occurs at the start of the XMP section
pub const XMP_MARKER: &[u8] = "http://ns.adobe.com/xap/1.0/".as_bytes();

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

/// Convert an XML Attribute in a node to a string, erroring if the attribute is missing.
///
/// # Arguments
/// * `node`: The node to read the attribute from
/// * `namespace`: The namespace of the attribute
/// * `attribute`: The name of the attribute
///
/// # Returns
///  Option holding the attribute converted to a string.
///
/// # Error
/// Will error if the attribute is not found.
fn attribute_to_str_req(
    node: Node,
    namespace: &str,
    attribute: &str,
) -> Result<String, GCameraError> {
    return attribute_to_str(node, namespace, attribute).ok_or(GCameraError::XMLMissingAttribute {
        attribute: String::from(attribute),
    });
}

/// Parse an attribute.
///
/// This parsing it using the str.parse method.
///
/// # Arguments
/// * `node`: The node to read the attribute from
/// * `namespace`: The namespace of the attribute
/// * `attribute`: The name of the attribute
///
/// # Returns
///  Option holding the parsed attribute, or an error.
fn parse_attribute<T: str::FromStr>(
    node: Node,
    namespace: &str,
    attribute: &str,
) -> Result<Option<T>, GCameraError> {
    let attrib_val = node.attribute((namespace, attribute));
    return attrib_val
        .map(|n| return n.parse())
        .transpose()
        .map_err(|_| {
            return GCameraError::XMLAttributeParseError {
                attribute: attrib_val.map(|n| return String::from(n)),
            };
        });
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Enumeration of different MIME Types that resources can have.
pub enum MimeType {
    /// JPEG Image
    Jpeg,

    ///MP4 Video
    Mp4,
}

impl TryFrom<String> for MimeType {
    type Error = GCameraError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        return match value.as_str() {
            "image/jpeg" => Ok(Self::Jpeg),
            "video/mp4" => Ok(Self::Mp4),
            _ => Err(GCameraError::UnknownMimeType { mime: value }),
        };
    }
}

/// Enumeration of possible semantic types for for resources in the XMP data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SemanticType {
    /// The main JPEG image
    Primary,

    /// Motion Photo Additional Resource
    MotionPhoto,

    /// A gain map, used for UltraHDR image formats
    GainMap,
}

/// Implementation to try to create a semantic enum from a string slice.
impl TryFrom<String> for SemanticType {
    type Error = GCameraError;

    /// Create a semantic enum from a string slice.
    ///
    /// # Arguments
    /// value: The value to attempt to create an enum from
    ///
    /// # Returns
    /// Result holding either the created enum, or an error.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        return match value.as_str() {
            "Primary" => Ok(Self::Primary),
            "MotionPhoto" => Ok(Self::MotionPhoto),
            "GainMap" => Ok(Self::GainMap),
            _ => Err(GCameraError::UnknownResourceSemantic { semantic: (value) }),
        };
    }
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

/// Implementation to create description from XML Node
impl TryFrom<Node<'_, '_>> for Description {
    type Error = GCameraError;

    /// Create an instance from the XML Element
    ///
    /// # Arguments
    /// * `xml_element`: The XML Node to create the description from.
    ///
    /// # Returns
    ///  Created description instance.
    fn try_from(xml_element: Node) -> Result<Self, Self::Error> {
        return Ok(Self {
            extended_xmp_id: attribute_to_str(xml_element, XMP_NOTE_NS, "HasExtendedXMP"),
            motion_photo: parse_attribute(xml_element, GCAMERA_NS, "MotionPhoto")?,
            motion_photo_version: parse_attribute(xml_element, GCAMERA_NS, "MotionPhotoVersion")?,
            motion_photo_timestamp_us: parse_attribute(
                xml_element,
                GCAMERA_NS,
                "MotionPhotoPresentationTimestampUs",
            )?,
        });
    }
}

/// Data about a single resource in the file
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Item {
    /// The mimetype of the resource
    pub mimetype: MimeType,

    /// The length of the resource
    /// Optional for primary resource.
    pub length: Option<usize>,

    /// Length in bytes between end of resource and start of next resource
    pub padding: usize,

    ///The semantic type of the resource.
    pub semantic: SemanticType,

    /// Optional Parameter to disambiguate items of the same semantic type
    pub label: Option<String>,

    /// Optional URI string containing relative URI of the item.
    /// Only present if the base media format is ISO/IEC 14496-12
    pub uri: Option<String>,
}

/// Implementation to create item from a XML Node.
impl TryFrom<Node<'_, '_>> for Item {
    type Error = GCameraError;

    fn try_from(value: Node<'_, '_>) -> Result<Self, Self::Error> {
        return Ok(Self {
            mimetype: MimeType::try_from(attribute_to_str_req(value, ITEM_NS, "Mime")?)?,
            length: parse_attribute(value, ITEM_NS, "Length")?,
            padding: parse_attribute(value, ITEM_NS, "Padding")?.unwrap_or(0),
            semantic: SemanticType::try_from(attribute_to_str_req(value, ITEM_NS, "Semantic")?)?,
            label: attribute_to_str(value, ITEM_NS, "Label"),
            uri: attribute_to_str(value, ITEM_NS, "URI"),
        });
    }
}

/// Struct holding data about resources as read from the XMP data.
#[derive(Debug, Eq, PartialEq)]
pub struct XMPData {
    /// The desscription of the XMP data
    pub description: Description,

    /// Vector of the resources defined in the file, according to the XMP data.
    pub resources: Vec<Item>,
}

impl XMPData {
    /// Create `JpegSegment` for XMP data that has no resources.
    ///
    /// # Returns
    /// Segment for XMP Resource that does not have any resources.
    pub fn as_resourceless_segment(&self) -> JpegSegment {
        // FIXME: Test for case with depth map

        let extended_xmp_note = match &self.description.extended_xmp_id {
            Some(note) => format!("\n      xmpNote:HasExtendedXMP=\"{note}\""),
            None => String::new(),
        };

        let xml_string = format!(
            "\
<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\">
  <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">
    <rdf:Description rdf:about=\"\"
      xmlns:xmpNote=\"http://ns.adobe.com/xmp/note/\"{extended_xmp_note}/>
  </rdf:RDF>
</x:xmpmeta>"
        );
        let data = [XMP_MARKER, &[0x00], xml_string.as_bytes()].concat();

        return JpegSegment::new(JpegMarker::APP1, &data);
    }
}

/// Implementation to create XMP Data from XML Document
impl TryFrom<Document<'_>> for XMPData {
    type Error = GCameraError;

    /// Create an instance from an XML Document.
    ///
    /// # Arguments
    /// * `document`: The XML Document to create the instance from.
    ///
    /// # Returns
    /// Instance created from the given XML Document
    fn try_from(document: Document) -> Result<Self, Self::Error> {
        let description_node = document.descendants().find(|n| {
            return n.tag_name() == ExpandedName::from((RDF_NS, "Description"));
        });

        if let Some(node) = description_node {
            let resource_nodes = document
                .descendants()
                .filter(|n| return n.tag_name() == ExpandedName::from((CONTAINER_NS, "Item")))
                .map(|n| return Item::try_from(n).unwrap()); // FIXME: Get rid of unwrap

            return Ok(Self {
                description: Description::try_from(node)?,
                resources: resource_nodes.collect(),
            });
        } else {
            return Err(GCameraError::DescriptionNodeNotFound);
        }
    }
}

/// Implementation to create XMP Data from string
impl TryFrom<String> for XMPData {
    type Error = GCameraError;

    /// Create an instance from the given string
    ///
    /// # Arguments
    ///  * `xmp_str`: The string to create the instance from
    ///
    /// # Returns
    /// Instance created from the given string
    fn try_from(xmp_string: String) -> Result<Self, Self::Error> {
        let xml_document = Document::parse(&xmp_string);

        match xml_document {
            Ok(document) => return Self::try_from(document),
            Err(typ) => {
                return Err(GCameraError::XMLParsingError { xml_error: typ });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests for the `attribute_to_str` function
    mod test_attribute_to_str {
        use super::*;

        /// Test the `attribute_to_str` function when it has a valid value.
        #[test]
        fn test_some() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.example.com", "a"),
                Some(String::from("Hello"))
            );
        }

        /// Test the `attribute_to_str` function when it does not have a valid value.
        #[test]
        fn test_none() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.example.com", "b"),
                None
            );
        }

        /// Test the `attribute_to_str` function when namespace is invalid.
        #[test]
        fn test_bad_ns() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                attribute_to_str(xml_element, "http://ns.second.example.com", "b"),
                None
            );
        }

        /// Test the `attribute_to_str` function when there are no namespaces
        #[test]
        fn test_no_ns() {
            let test_xml = "<tagname a=\"Hello\"/>";
            let document = Document::parse(test_xml).unwrap();
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

    /// Tests for the `parse_attribute` method
    mod test_parse_attribute {

        use super::*;

        /// Test the `parse_attribute` function when it has a valid value.
        #[test]
        fn test_some() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                parse_attribute(xml_element, "http://ns.example.com", "a"),
                Ok(Some(1))
            );
        }

        /// Test the `parse_attribute` function when it does not have a valid value.
        #[test]
        fn test_none() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                parse_attribute::<u32>(xml_element, "http://ns.example.com", "b"),
                Ok(None)
            );
        }

        /// Test the `parse_attribute` function when namespace is invalid.
        #[test]
        fn test_bad_ns() {
            let test_xml = "<tagname t:a=\"1\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                parse_attribute::<u32>(xml_element, "http://ns.second.example.com", "b"),
                Ok(None)
            );
        }

        /// Test the `parse_attribute` function when there are no namespaces
        #[test]
        fn test_no_ns() {
            let test_xml = "<tagname a=\"1\"/>";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();

            assert_eq!(
                parse_attribute::<u32>(xml_element, "http://ns.example.com", "b"),
                Ok(None)
            );
        }

        /// Test case where the attribute cannot be parsed to a string.
        #[test]
        fn test_not_parseable() {
            let test_xml = "<tagname t:a=\"Hello\" xmlns:t=\"http://ns.example.com\" />";
            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "tagname")
                .unwrap();
            let parse_result = parse_attribute::<u32>(xml_element, "http://ns.example.com", "a");

            assert_eq!(
                parse_result,
                Err(GCameraError::XMLAttributeParseError {
                    attribute: Some(String::from("Hello"))
                })
            );
        }
    }

    /// Tests for the `MimeType` enum
    mod mime_type_test {
        use super::*;

        /// Test the `try_from` method
        #[test]
        fn test_try_from() {
            let cases = vec![
                (String::from("image/jpeg"), MimeType::Jpeg),
                (String::from("video/mp4"), MimeType::Mp4),
            ];

            for (input, expected) in cases {
                assert_eq!(MimeType::try_from(input), Ok(expected));
            }
        }

        /// Test `try_from` when the mimetype is not known by the tool
        #[test]
        fn test_try_from_bad_string() {
            assert_eq!(
                MimeType::try_from(String::from("Hello")),
                Err(GCameraError::UnknownMimeType {
                    mime: String::from("Hello")
                })
            );
        }
    }

    /// Tests for the `SemanticType` enum
    mod semantic_type_test {
        use super::*;

        /// Test the `try_from` method
        #[test]
        fn test_try_from() {
            let cases = vec![
                (String::from("Primary"), SemanticType::Primary),
                (String::from("MotionPhoto"), SemanticType::MotionPhoto),
                (String::from("GainMap"), SemanticType::GainMap),
            ];

            for (input, expected) in cases {
                assert_eq!(SemanticType::try_from(input), Ok(expected));
            }
        }

        /// Test `try_from` with a bad string.
        #[test]
        fn test_try_from_bad_string() {
            assert_eq!(
                SemanticType::try_from(String::from("Hello")),
                Err(GCameraError::UnknownResourceSemantic {
                    semantic: String::from("Hello")
                })
            );
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

            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "Description")
                .unwrap();
            let description = Description::try_from(xml_element);

            assert_eq!(
                description,
                Ok(Description {
                    extended_xmp_id: Some(String::from("DD558CA2166AEC119A42CDFB02D4F1EF")),
                    motion_photo: Some(1),
                    motion_photo_version: Some(1),
                    motion_photo_timestamp_us: Some(968644),
                }),
            );
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
              Item:Padding=\"1\"
              xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"
              xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"/>";

            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "Item")
                .unwrap();
            let item = Item::try_from(xml_element);

            assert_eq!(
                item,
                Ok(Item {
                    mimetype: MimeType::Mp4,
                    length: Some(4906025),
                    padding: 1,
                    semantic: SemanticType::MotionPhoto,
                    label: None,
                    uri: None
                })
            );
        }

        /// Tests initializing without the "padding" member
        #[test]
        fn test_init_no_padding() {
            let test_xml = "<Container:Item
              Item:Mime=\"video/mp4\"
              Item:Semantic=\"MotionPhoto\"
              Item:Length=\"4906025\"
              xmlns:Container=\"http://ns.google.com/photos/1.0/container/\"
              xmlns:Item=\"http://ns.google.com/photos/1.0/container/item/\"/>";

            let document = Document::parse(test_xml).unwrap();
            let xml_element = document
                .descendants()
                .find(|n| return n.tag_name().name() == "Item")
                .unwrap();
            let item = Item::try_from(xml_element);

            assert_eq!(
                item,
                Ok(Item {
                    mimetype: MimeType::Mp4,
                    length: Some(4906025),
                    padding: 0,
                    semantic: SemanticType::MotionPhoto,
                    label: None,
                    uri: None
                })
            );
        }
    }

    /// Tests for the `XMPData` struct
    mod test_xmp_data {
        use super::*;

        /// Test the `try_from` method from parsing from XML Document
        #[test]
        fn test_from_xml() {
            let document = Document::parse(
                "<x:xmpmeta xmlns:x='adobe:ns:meta/' x:xmptk='Adobe XMP Core 5.1.0-jc003'>
                <rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>
                    <rdf:Description rdf:about=''
                    xmlns:xmpNote='http://ns.adobe.com/xmp/note/'
                    xmlns:GCamera='http://ns.google.com/photos/1.0/camera/'
                    xmlns:Container='http://ns.google.com/photos/1.0/container/'
                    xmlns:Item='http://ns.google.com/photos/1.0/container/item/'
                    xmpNote:HasExtendedXMP='DD558CA2166AEC119A42CDFB02D4F1EF'
                    GCamera:MotionPhoto='1'
                    GCamera:MotionPhotoVersion='1'
                    GCamera:MotionPhotoPresentationTimestampUs='968644'>
                    <Container:Directory>
                        <rdf:Seq>
                        <rdf:li rdf:parseType='Resource'>
                            <Container:Item
                            Item:Mime='image/jpeg'
                            Item:Semantic='Primary'
                            Item:Length='0'
                            Item:Padding='0' />
                        </rdf:li>
                        <rdf:li rdf:parseType='Resource'>
                            <Container:Item
                            Item:Mime='video/mp4'
                            Item:Semantic='MotionPhoto'
                            Item:Length='4906025'
                            Item:Padding='0' />
                        </rdf:li>
                        </rdf:Seq>
                    </Container:Directory>
                    </rdf:Description>
                </rdf:RDF>
                </x:xmpmeta>",
            )
            .unwrap();

            let data = XMPData::try_from(document);

            assert_eq!(
                data,
                Ok(XMPData {
                    description: Description {
                        extended_xmp_id: Some(String::from("DD558CA2166AEC119A42CDFB02D4F1EF")),
                        motion_photo: Some(1),
                        motion_photo_version: Some(1),
                        motion_photo_timestamp_us: Some(968644),
                    },
                    resources: vec![
                        Item {
                            mimetype: MimeType::Jpeg,
                            semantic: SemanticType::Primary,
                            length: Some(0),
                            padding: 0,
                            uri: None,
                            label: None,
                        },
                        Item {
                            mimetype: MimeType::Mp4,
                            semantic: SemanticType::MotionPhoto,
                            length: Some(4906025),
                            padding: 0,
                            uri: None,
                            label: None,
                        },
                    ],
                },)
            );
        }

        /// Test for when there is no description node
        #[test]
        fn test_no_description() {
            let document = Document::parse(
                "<x:xmpmeta xmlns:x='adobe:ns:meta/' x:xmptk='Adobe XMP Core 5.1.0-jc003'>
                <rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>
                </rdf:RDF>
                </x:xmpmeta>",
            )
            .unwrap();

            let data = XMPData::try_from(document);

            assert_eq!(data, Err(GCameraError::DescriptionNodeNotFound));
        }

        /// Basic test for the `from_str` method
        #[test]
        fn test_from_str() {
            let xml_string = String::from(
                "<x:xmpmeta xmlns:x='adobe:ns:meta/' x:xmptk='Adobe XMP Core 5.1.0-jc003'>
                <rdf:RDF xmlns:rdf='http://www.w3.org/1999/02/22-rdf-syntax-ns#'>
                    <rdf:Description rdf:about=''
                    xmlns:xmpNote='http://ns.adobe.com/xmp/note/'
                    xmlns:GCamera='http://ns.google.com/photos/1.0/camera/'
                    xmlns:Container='http://ns.google.com/photos/1.0/container/'
                    xmlns:Item='http://ns.google.com/photos/1.0/container/item/'
                    xmpNote:HasExtendedXMP='DD558CA2166AEC119A42CDFB02D4F1EF'
                    GCamera:MotionPhoto='1'
                    GCamera:MotionPhotoVersion='1'
                    GCamera:MotionPhotoPresentationTimestampUs='968644'>
                    <Container:Directory>
                        <rdf:Seq>
                        <rdf:li rdf:parseType='Resource'>
                            <Container:Item
                            Item:Mime='image/jpeg'
                            Item:Semantic='Primary'
                            Item:Length='0'
                            Item:Padding='0' />
                        </rdf:li>
                        <rdf:li rdf:parseType='Resource'>
                            <Container:Item
                            Item:Mime='video/mp4'
                            Item:Semantic='MotionPhoto'
                            Item:Length='4906025'
                            Item:Padding='0' />
                        </rdf:li>
                        </rdf:Seq>
                    </Container:Directory>
                    </rdf:Description>
                </rdf:RDF>
                </x:xmpmeta>",
            );

            let data = XMPData::try_from(xml_string);

            assert_eq!(
                data,
                Ok(XMPData {
                    description: Description {
                        extended_xmp_id: Some(String::from("DD558CA2166AEC119A42CDFB02D4F1EF")),
                        motion_photo: Some(1),
                        motion_photo_version: Some(1),
                        motion_photo_timestamp_us: Some(968644),
                    },
                    resources: vec![
                        Item {
                            mimetype: MimeType::Jpeg,
                            semantic: SemanticType::Primary,
                            length: Some(0),
                            padding: 0,
                            uri: None,
                            label: None,
                        },
                        Item {
                            mimetype: MimeType::Mp4,
                            semantic: SemanticType::MotionPhoto,
                            length: Some(4906025),
                            padding: 0,
                            uri: None,
                            label: None,
                        },
                    ],
                }),
            );
        }

        /// Test case for invalid string
        #[test]
        fn test_from_str_invalid() {
            let test_str = String::from("Hello World");
            let parsed = XMPData::try_from(test_str);

            assert!(parsed.is_err());
            assert!(matches!(
                parsed.unwrap_err(),
                GCameraError::XMLParsingError { .. }
            ));
        }

        /// Test the `as_resourceless_segment` method when there is extended XMP data
        #[test]
        fn test_resourceless_segment_with_extended() {
            let expected_data_bytes = "\
http://ns.adobe.com/xap/1.0/\x00<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\">
  <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">
    <rdf:Description rdf:about=\"\"
      xmlns:xmpNote=\"http://ns.adobe.com/xmp/note/\"
      xmpNote:HasExtendedXMP=\"ABCDEFG\"/>
  </rdf:RDF>
</x:xmpmeta>".as_bytes();
            let data = XMPData {
                description: Description {
                    extended_xmp_id: Some(String::from("ABCDEFG")),
                    motion_photo: Some(1),
                    motion_photo_version: Some(1),
                    motion_photo_timestamp_us: Some(5),
                },
                resources: Vec::new(),
            };
            let created_segment = data.as_resourceless_segment();
            assert_eq!(
                created_segment,
                JpegSegment::new(JpegMarker::APP1, expected_data_bytes)
            );
        }

        /// Test the `as_resourceless_segment` method when there is no extended XMP data
        #[test]
        fn test_resourceless_segment_without_extended() {
            let expected_data_bytes = "\
http://ns.adobe.com/xap/1.0/\x00<x:xmpmeta xmlns:x=\"adobe:ns:meta/\" x:xmptk=\"Adobe XMP Core 5.1.0-jc003\">
  <rdf:RDF xmlns:rdf=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#\">
    <rdf:Description rdf:about=\"\"
      xmlns:xmpNote=\"http://ns.adobe.com/xmp/note/\"/>
  </rdf:RDF>
</x:xmpmeta>".as_bytes();
            let data = XMPData {
                description: Description {
                    extended_xmp_id: None,
                    motion_photo: Some(1),
                    motion_photo_version: Some(1),
                    motion_photo_timestamp_us: Some(5),
                },
                resources: Vec::new(),
            };
            let created_segment = data.as_resourceless_segment();
            assert_eq!(
                created_segment,
                JpegSegment::new(JpegMarker::APP1, expected_data_bytes)
            );
        }
    }
}
