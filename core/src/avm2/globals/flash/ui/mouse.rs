//! `flash.ui.Mouse` builtin/prototype

use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::traits::Trait;
use crate::avm2::{Activation, Error, Namespace, Object, QName, Value};
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.ui.Mouse`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.ui.Mouse`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `hide`
pub fn hide<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    activation.context.ui.set_mouse_visible(false);

    Ok(Value::Undefined)
}

/// Implements `show`
pub fn show<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    activation.context.ui.set_mouse_visible(true);

    Ok(Value::Undefined)
}

/// Implements `registerCursor`
pub fn register_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("flash.ui.Mouse.registerCursor(), not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `unregisterCursor`
pub fn unregister_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("flash.ui.Mouse.unregisterCursor(), not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `cursor` property getter
pub fn cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("flash.ui.Mouse.cursor get(), not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `cursor` property setter
pub fn set_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("flash.ui.Mouse.cursor set(), not yet implemented");

    Ok(Value::Undefined)
}

/// Implements `supportsCursor` property getter
pub fn supports_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(true.into())
}

/// Implements `supportsNativeCursor` property getter
pub fn supports_native_cursor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(true.into())
}

/// Construct `Mouse`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "Mouse"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    write.define_class_trait(Trait::from_getter(
        QName::new(Namespace::public(), "cursor"),
        Method::from_builtin(cursor)
    ));
    write.define_class_trait(Trait::from_setter(
        QName::new(Namespace::public(), "cursor"),
        Method::from_builtin(set_cursor)
    ));
    write.define_class_trait(Trait::from_getter(
        QName::new(Namespace::public(), "supportsCursor"),
        Method::from_builtin(supports_cursor)
    ));
    write.define_class_trait(Trait::from_getter(
        QName::new(Namespace::public(), "supportsNativeCursor"),
        Method::from_builtin(supports_native_cursor)
    ));
    write.define_class_trait(Trait::from_method(
        QName::new(Namespace::public(), "hide"),
        Method::from_builtin(hide)
    ));
    write.define_class_trait(Trait::from_method(
        QName::new(Namespace::public(), "show"),
        Method::from_builtin(show)
    ));
    write.define_class_trait(Trait::from_method(
        QName::new(Namespace::public(), "registerCursor"),
        Method::from_builtin(register_cursor)
    ));
    write.define_class_trait(Trait::from_method(
        QName::new(Namespace::public(), "unregisterCursor"),
        Method::from_builtin(unregister_cursor)
    ));

    class
}
