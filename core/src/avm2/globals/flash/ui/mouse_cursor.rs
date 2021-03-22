//! `flash.ui.MouseCursor` builtin/prototype

use crate::avm1::AvmString;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::Method;
use crate::avm2::traits::Trait;
use crate::avm2::{Activation, Error, Namespace, Object, QName, Value};
use gc_arena::{GcCell, MutationContext};
use crate::avm2::names::Multiname;

/// Implements `flash.ui.MouseCursor`'s instance constructor.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `flash.ui.MouseCursor`'s class initializer.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}


/// Construct `MouseCursor`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.ui"), "MouseCursor"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);
    write.set_attributes(ClassAttributes::SEALED);

    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "ARROW"),
        Multiname::from(QName::new(Namespace::public(), "String")),
        Some(AvmString::new(mc, "arrow").into())
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "AUTO"),
        Multiname::from(QName::new(Namespace::public(), "String")),
        Some(AvmString::new(mc, "auto").into())
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "BUTTON"),
        Multiname::from(QName::new(Namespace::public(), "String")),
        Some(AvmString::new(mc, "button").into())
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "HAND"),
        Multiname::from(QName::new(Namespace::public(), "String")),
        Some(AvmString::new(mc, "hand").into())
    ));
    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "IBEAM"),
        Multiname::from(QName::new(Namespace::public(), "String")),
        Some(AvmString::new(mc, "ibeam").into())
    ));

    class
}
