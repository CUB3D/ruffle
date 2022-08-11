use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::as_broadcaster::BroadcasterFunctions;
use crate::avm1::object::file_reference::FileReferenceObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::avm_warn;
use crate::backend::ui::FileFilter;
use crate::string::AvmString;
use gc_arena::MutationContext;
use url::Url;

// There are two undocumented functions in FileReference: convertToPPT and deleteConvertedPPT.
// Until further reason is given, they will be unimplemented.
// See:
// ASSetPropFlags(flash.net.FileReference.prototype, null, 6, 1);
// for(var k in flash.net.FileReference.prototype) {
// 	trace(k);
// }

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "creationDate" => property(creation_date);
    "creator" => property(creator);
    "modificationDate" => property(modification_date);
    "name" => property(name);
    "postData" => property(post_data, set_post_data);
    "size" => property(size);
    "type" => property(file_type);
    "browse" => method(browse; DONT_ENUM);
    "cancel" => method(cancel; DONT_ENUM);
    "download" => method(download; DONT_ENUM);
    "upload" => method(upload; DONT_ENUM);
};

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.into())
}

pub fn creation_date<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref
            .creation_date()
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn creator<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.creator().map_or(Value::Undefined, |x| {
            AvmString::new_utf8(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn modification_date<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref
            .modification_date()
            .map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn name<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.name().map_or(Value::Undefined, |x| {
            AvmString::new_utf8(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn post_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(AvmString::new_utf8(activation.context.gc_context, file_ref.post_data()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_post_data<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let post_data = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    if let Some(file_ref) = this.as_file_reference_object() {
        file_ref.set_post_data(activation.context.gc_context, post_data.to_string());
    }

    Ok(Value::Undefined)
}

pub fn size<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.size().map_or(Value::Undefined, |x| x.into()));
    }

    Ok(Value::Undefined)
}

pub fn file_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_ref) = this.as_file_reference_object() {
        return Ok(file_ref.file_type().map_or(Value::Undefined, |x| {
            AvmString::new_utf8(activation.context.gc_context, x).into()
        }));
    }

    Ok(Value::Undefined)
}

pub fn browse<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let file_filters = match args.get(0) {
        Some(Value::Object(array)) => {
            // Array of filter objects.
            let length = array.length(activation)?;

            // Empty array is not allowed
            if length == 0 {
                return Ok(false.into());
            }

            let mut results = Vec::with_capacity(length as usize);

            for i in 0..length {
                if let Value::Object(element) = array.get_element(activation, i) {
                    let mac_type = if let Ok(val) = element.get("macType", activation) {
                        Some(val.coerce_to_string(activation)?.to_string())
                    } else {
                        None
                    };

                    let description = element
                        .get("description", activation)?
                        .coerce_to_string(activation)?
                        .to_string();

                    let extensions = element
                        .get("extension", activation)?
                        .coerce_to_string(activation)?
                        .to_string();

                    // Empty strings are not allowed for desc / extension
                    if description.is_empty() || extensions.is_empty() {
                        return Ok(false.into());
                    }

                    results.push(FileFilter {
                        description,
                        extensions,
                        mac_type,
                    });
                } else {
                    return Err(Error::ThrownValue("Unexpected filter value".into()));
                }
            }

            results
        }
        None => Vec::new(),
        _ => return Ok(Value::Undefined),
    };

    let dialog = activation
        .context
        .ui
        .display_file_open_dialog(file_filters, false);

    let result = match dialog {
        Some(dialog) => {
            let process = activation.context.load_manager.select_file_dialog(
                activation.context.player.clone(),
                this,
                dialog,
            );

            activation.context.navigator.spawn_future(process);
            true
        }
        None => false,
    };

    Ok(result.into())
}

pub fn cancel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm_warn!(activation, "FileReference.cancel() not implemented");
    Ok(Value::Undefined)
}

pub fn download<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(url) = args.first() {
        let url_string = url.coerce_to_string(activation)?.to_string();

        // Invalid domain should bail out with false
        let url = match Url::parse(&url_string) {
            Ok(url) => url,
            Err(_) => return Ok(false.into()),
        };

        let file_name = match args.get(1) {
            Some(file_name) => file_name.coerce_to_string(activation)?.to_string(),
            None => {
                // Try to get the end of the path as a file name, if we can't bail and return false
                match url.path().split('/').last() {
                    Some(path_end) => path_end.to_string(),
                    None => return Ok(false.into()),
                }
            }
        };

        let domain = url.domain().unwrap_or("<unknown domain>").to_string();

        // Create and spawn dialog
        let dialog = activation.context.ui.display_file_save_dialog(
            file_name,
            format!("Select location for download from {}", domain),
        );
        let result = match dialog {
            Some(dialog) => {
                let process = activation.context.load_manager.download_file_dialog(
                    activation.context.player.clone(),
                    this,
                    dialog,
                    url_string,
                );

                activation.context.navigator.spawn_future(process);
                true
            }
            None => false,
        };

        return Ok(result.into());
    }

    Ok(false.into())
}

pub fn upload<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(file_reference) = this.as_file_reference_object() {
        // If we haven't `.browse()`ed something yet, we can't upload it
        if !file_reference.initialised() {
            return Ok(false.into());
        }

        if let Some(url) = args.first() {
            let url_string = url.coerce_to_string(activation)?.to_string();

            // Invalid domain should bail out with false
            let url = match Url::parse(&url_string) {
                Ok(url) => url,
                Err(_) => return Ok(false.into()),
            };

            // We should only allow uploads to http(s) urls
            match url.scheme() {
                "https" | "http" => {}
                _ => return Ok(false.into()),
            }

            let process = activation.context.load_manager.upload_file(
                activation.context.player.clone(),
                this,
                url_string,
                file_reference.data(),
                file_reference.name().unwrap_or_else(|| "file".to_string()),
            );

            activation.context.navigator.spawn_future(process);

            return Ok(true.into());
        }

        return Ok(false.into());
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
    array_proto: Object<'gc>,
    broadcaster_functions: BroadcasterFunctions<'gc>,
) -> Object<'gc> {
    let object = FileReferenceObject::empty_object(gc_context, Some(proto));
    broadcaster_functions.initialize(gc_context, object.into(), array_proto);
    let script_object = object.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, script_object, fn_proto);
    object.into()
}
