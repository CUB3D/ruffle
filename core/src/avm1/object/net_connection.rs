use crate::{add_field_accessors, loader};
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext, CollectionContext};

use std::fmt;
use std::net::{TcpStream, SocketAddr};
use std::rc::Rc;
use crate::backend::navigator::OwnedFuture;
use crate::avm1::error::Error;
use rml_rtmp::handshake::{Handshake, PeerType, HandshakeProcessResult};
use rml_rtmp::chunk_io::{ChunkDeserializer, ChunkSerializer, Packet};
use rml_amf0::Amf0Value;
use std::collections::HashMap;
use rml_rtmp::messages::{RtmpMessage, UserControlEventType};
use rml_rtmp::time::RtmpTimestamp;
use std::io::{ErrorKind, Read, Write};
use std::cell::RefCell;
use std::sync::{Mutex, Arc};
use crate::backend::tcp::TcpBackend;

/// A NetConnection
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct NetConnectionObject<'gc>(GcCell<'gc, NetConnectionData<'gc>>);

#[derive(Clone/*, Collect*/)]
// #[collect(no_drop)]
pub struct NetConnectionData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,
    queue: Arc<Mutex<Vec<Packet>>>,
}

impl fmt::Debug for NetConnectionObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("NetConnection")
            .finish()
    }
}

unsafe impl<'gc> Collect for NetConnectionData<'gc> {

}

impl<'gc> NetConnectionObject<'gc> {
    // add_field_accessors!();

    pub fn send_all(&self, gc_context: MutationContext<'gc, '_>, bytes: Packet) {
        let queue = &self.0.write(gc_context).queue;
        let mut q = queue.lock().unwrap();
        q.push(bytes);
    }


    pub fn connect(&self, gc_context: MutationContext<'gc, '_>, addr: SocketAddr, app_name: &str, page_url: String, connection_string: String, version_string: String, tcp: &mut dyn TcpBackend) -> Option<OwnedFuture<(), loader::Error>> {
        if let Some(mut tcp_handle) = tcp.connect(addr) {
            let queue = self.0.read().queue.clone();
            let app_name = app_name.to_string();
            Some(Box::pin(async move {
                let mut buf = [0u8; 4096];

                let mut cs = ChunkSerializer::new();
                let mut decoder = ChunkDeserializer::new();

                let mut hand = Handshake::new(PeerType::Client);
                tcp_handle.write_all(&hand.generate_outbound_p0_and_p1().expect("p0p1"));

                // Process the handshake
                loop {
                    match tcp_handle.try_read(&mut buf) {
                        Some(len) => {
                            let hand = hand.process_bytes(&buf[..len]).expect("handshake");
                            match hand {
                                HandshakeProcessResult::InProgress { response_bytes } => {
                                    tcp_handle.write_all(&response_bytes);
                                }
                                HandshakeProcessResult::Completed { response_bytes, remaining_bytes } => {
                                    tcp_handle.write_all(&response_bytes);
                                    //TODO: handle remaining
                                    // decoder.get_next_message()

                                    break;
                                }
                            }
                        }
                        None => {}
                    }
                }

                // connect packet
                let mut properties = HashMap::new();
                // Name of app to connect to
                properties.insert("app".to_string(), Amf0Value::Utf8String(app_name));
                // Flash version
                properties.insert("flashVer".to_string(), Amf0Value::Utf8String(version_string));
                // Url to swf
                properties.insert("swfUrl".to_string(), Amf0Value::Utf8String(page_url));
                properties.insert("tcUrl".to_string(), Amf0Value::Utf8String(connection_string));
                // Proxy use
                properties.insert("fpad".to_string(), Amf0Value::Boolean(false));
                properties.insert("capabilities".to_string(), Amf0Value::Number(329.0));
                // SUPPORT_SND_SPEEX | SUPPORT_SND_G711U | SUPPORT_SND_G711A | SUPPORT_SND_NELLY | SUPPORT_SND_NELLY8 | SUPPORT_SND_UNUSED | SUPPORT_SND_MP3 | SUPPORT_SND_ADPCM | SUPPORT_SND_NONE
                properties.insert("audioCodecs".to_string(), Amf0Value::Number(3575.0));
                // SUPPORT_VID_SORENSON | SUPPORT_VID_HOMEBREW | SUPPORT_VID_VP6 | SUPPORT_VID_VP6ALPHA | SUPPORT_VID_HOMEBREWV | SUPPORT_VID_H264
                properties.insert("videoCodecs".to_string(), Amf0Value::Number(252.0));
                // SUPPORT_VID_CLIENT_SEEK
                properties.insert("videoFunction".to_string(), Amf0Value::Number(1.0));
                // Url of current page
                properties.insert("pageUrl".to_string(), Amf0Value::Undefined);
                let message = RtmpMessage::Amf0Command {
                    command_name: "connect".to_string(),
                    command_object: Amf0Value::Object(properties),
                    additional_arguments: vec![],
                    transaction_id: 1 as f64,
                };
                let payload = message.into_message_payload(RtmpTimestamp::new(0), 0).expect("Failed to msg -> paylaod");
                let ser = cs.serialize(&payload, false, false).expect("Failed to serial");
                queue.lock().unwrap().push(ser);

                // Main socket loop
                loop {
                    match tcp_handle.try_read(&mut buf) {
                        Some(len) => {
                            println!("processing {:x?}", &buf[..len]);
                            if let Ok(Some(msg)) = decoder.get_next_message(&buf[..len]) {
                                let msg = msg.to_rtmp_message().expect("pay -> msg");
                                match msg {
                                    RtmpMessage::UserControl { stream_id, buffer_length, ref event_type, timestamp } => {
                                        match event_type {
                                            UserControlEventType::PingRequest => {
                                                let msg = RtmpMessage::UserControl {
                                                    timestamp,
                                                    event_type: UserControlEventType::PingRequest,
                                                    buffer_length: None,
                                                    stream_id,
                                                };
                                                let payload = msg.into_message_payload(RtmpTimestamp::new(0), 0).expect("msg -> pay");
                                                let ser = cs.serialize(&payload, false, false).expect("ping");
                                                queue.lock().unwrap().push(ser);
                                            }
                                            _ => {}
                                        }
                                    }
                                    RtmpMessage::Amf0Command { command_name, command_object, transaction_id, additional_arguments } => {
                                        println!("Got call reponse: {} {:?} {} {:?}", command_name, command_object, transaction_id, additional_arguments);
                                    }
                                    _ => {}
                                }
                            }
                        },
                        None => {}
                    }

                    let mut q = queue.lock().unwrap();
                    while let Some(packet) = q.pop() {
                        tcp_handle.write_all(&packet.bytes);
                    }
                }

                Ok(())
            }))
        } else {
            None
        }
    }

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        NetConnectionObject(GcCell::allocate(
            gc_context,
            NetConnectionData {
                base: ScriptObject::object(gc_context, proto),
                queue: Arc::new(Mutex::new(Vec::new()))
            },
        ))
    }
}

impl<'gc> TObject<'gc> for NetConnectionObject<'gc> {
    impl_custom_object!(base {
        set(proto: net_connection);
        bare_object(as_net_connection_object -> NetConnectionObject::empty_object);
    });
}
