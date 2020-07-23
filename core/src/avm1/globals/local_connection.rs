use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::property::Attribute;
use crate::avm1::{AvmString, Object, ScriptObject, TObject, UpdateContext, Value};
use crate::backend::navigator::{NavigationMethod, RequestOptions};
use gc_arena::MutationContext;
use std::borrow::Cow;
use enumset::EnumSet;

/// Implements `LocalConnection`
pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No-op constructor
    Ok(Value::Undefined)
}

pub fn close<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("LocalConnection.close() is not implemented");
    Ok(Value::Undefined)
}

pub fn connect<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("LocalConnection.connect() is not implemented");
    Ok(Value::Bool(false))
}

pub fn domain<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::String(AvmString::new(_context.gc_context, "localhost")))
}

pub fn send<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("LocalConnection.send() is not implemented");
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    //TODO: attrs and order, also are events part of proto?
    object.force_set_function("close", close, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("connect", connect, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("domain", domain, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("send", send, gc_context, EnumSet::empty(), Some(fn_proto));


    object.into()
}

