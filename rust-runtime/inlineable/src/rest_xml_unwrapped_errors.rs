/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Error abstractions for `noErrorWrapping`. Code generators should either inline this file
//! or its companion `rest_xml_wrapped_errors.rs` for code generation

use aws_smithy_xml::decode::{try_data, Document, ScopedDecoder, XmlDecodeError};
use std::convert::TryFrom;

#[allow(unused)]
pub fn body_is_error(body: &[u8]) -> Result<bool, XmlDecodeError> {
    let mut doc = Document::try_from(body)?;
    let scoped = doc.root_element()?;
    Ok(scoped.start_el().matches("Error"))
}

pub fn error_scope<'a, 'b>(
    doc: &'a mut Document<'b>,
) -> Result<ScopedDecoder<'b, 'a>, XmlDecodeError> {
    let scoped = doc.root_element()?;
    if !scoped.start_el().matches("Error") {
        return Err(XmlDecodeError::custom("expected error as root"));
    }
    Ok(scoped)
}

pub fn parse_generic_error(body: &[u8]) -> Result<aws_smithy_types::Error, XmlDecodeError> {
    let mut doc = Document::try_from(body)?;
    let mut root = doc.root_element()?;
    let mut err = aws_smithy_types::Error::builder();
    while let Some(mut tag) = root.next_tag() {
        match tag.start_el().local() {
            "Code" => {
                err.code(try_data(&mut tag)?);
            }
            "Message" => {
                err.message(try_data(&mut tag)?);
            }
            "RequestId" => {
                err.request_id(try_data(&mut tag)?);
            }
            _ => {}
        }
    }
    Ok(err.build())
}

#[cfg(test)]
mod test {
    use super::{body_is_error, parse_generic_error};

    #[test]
    fn parse_unwrapped_error() {
        let xml = br#"<Error>
    <Type>Sender</Type>
    <Code>InvalidGreeting</Code>
    <Message>Hi</Message>
    <AnotherSetting>setting</AnotherSetting>
    <RequestId>foo-id</RequestId>
</Error>"#;
        assert!(body_is_error(xml).unwrap());
        let parsed = parse_generic_error(xml).expect("valid xml");
        assert_eq!(parsed.request_id(), Some("foo-id"));
        assert_eq!(parsed.message(), Some("Hi"));
        assert_eq!(parsed.code(), Some("InvalidGreeting"));
    }
}
