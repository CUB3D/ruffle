//! NetConnection object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::object::bevel_filter::{BevelFilterObject, BevelFilterType};
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{AvmString, Object, TObject, Value, ScriptObject};
use gc_arena::MutationContext;
use rml_rtmp::sessions::ClientSessionConfig;
use rml_rtmp::messages::{RtmpMessage, UserControlEventType};
use std::collections::HashMap;
use rml_rtmp::time::RtmpTimestamp;
use rml_rtmp::chunk_io::{ChunkSerializer, ChunkDeserializer};
use std::net::{TcpStream, SocketAddr, IpAddr};
use rml_rtmp::handshake::{Handshake, PeerType, HandshakeProcessResult};
use std::io::{Write, Read, ErrorKind};
use crate::context_menu::ContextMenuCallback::Print;
use rml_amf0::Amf0Value;
use crate::avm1::object::net_connection::NetConnectionObject;
use url::Url;
use std::str::FromStr;
use std::convert::TryFrom;
use crate::display_object::TDisplayObject;

//TODO: contenttype property

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "call" => method(call);
    "connect" => method(connect);
};

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn call<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: how does missing arg work
    let function_name = args.get(0).unwrap_or(&Value::Undefined).coerce_to_string(activation)?.to_string();

    // rpc call packet
    let message = RtmpMessage::Amf0Command {
        command_name: function_name.clone(),
        command_object: Amf0Value::Object(HashMap::new()),
        additional_arguments: vec![],
        //TODO:
        transaction_id: 100 as f64,
    };
    let mut cs = ChunkSerializer::new();
    let payload = message.into_message_payload(RtmpTimestamp::new(0), 0).expect("Failed to msg -> paylaod");
    let ser = cs.serialize(&payload, false, false).expect("Failed to serial");

    if let Some(net_connection) = this.as_net_connection_object() {
        net_connection.send_all(activation.context.gc_context, ser);
    }

    Ok(Value::Undefined)
}

#[derive(Copy, Clone, Debug)]
enum Protocol {
    /// Real-Time Messaging Protocol
    RTMP,
}

impl TryFrom<&str> for Protocol {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "rtmp" => Ok(Self::RTMP),
            _ => Err(())
        }
    }
}

impl Protocol {
    fn default_port(&self) -> u16 {
        match self {
            Self::RTMP => 1935
        }
    }
}

pub fn connect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let target_uri = args.get(0).unwrap_or(&Value::Undefined).coerce_to_string(activation)?;

    if let Ok(uri) = Url::parse(target_uri.as_str()) {
        println!("url = {:?}", uri);
        if let Ok(protocol) = Protocol::try_from(uri.scheme()) {
            println!("protocol = {:?}", protocol);
            if let Some(obj) = this.as_net_connection_object() {

                let page_url = activation.target_clip_or_root().map(|tc| tc.as_movie_clip()
                    .and_then(|mc| mc.movie())
                    .and_then(|mov| mov.url().map(|s| s.to_string()))
                    .unwrap_or_else(|| "".to_string())
                ).unwrap_or_else(|_| "".to_string());
                println!("pu = {}", page_url);

                // todo: unwrap
                let addr = *uri.socket_addrs(|| Some(protocol.default_port())).expect("socket addr").first().unwrap();
                if let Some(fut) = obj.connect(activation.context.gc_context, addr, uri.path(), page_url, target_uri.to_string(), activation.context.system.get_version_string(activation.context.avm1), activation.context.tcp) {
                    activation.context.navigator.spawn_future(fut);
                    return Ok(true.into())
                } else {
                    return Ok(false.into())
                }
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = NetConnectionObject::empty_object(gc_context, Some(proto));
    let script_object = object.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, script_object, fn_proto);
    object.into()
}
