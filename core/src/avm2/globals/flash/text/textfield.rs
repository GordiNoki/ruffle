//! `flash.text.TextField` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::Method;
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{Object, TObject};
use crate::avm2::string::AvmString;
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::{AutoSizeMode, EditText, TDisplayObject};
use crate::html::TextFormat;
use crate::tag_utils::SwfMovie;
use crate::vminterface::AvmType;
use gc_arena::{GcCell, MutationContext};
use std::sync::Arc;

/// Implements `flash.text.TextField`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_display_object().is_none() {
            let movie = Arc::new(SwfMovie::empty(activation.context.swf.version()));
            let new_do = EditText::new(
                &mut activation.context,
                movie.clone(),
                0.0,
                0.0,
                100.0,
                100.0,
            );

            let movie_library = activation.context.library.library_for_movie_mut(movie);
            movie_library.check_avm_type(AvmType::Avm2).unwrap();

            this.init_display_object(activation.context.gc_context, new_do.into());
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.text.TextField`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

pub fn autosize<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(match this.autosize() {
            AutoSizeMode::None => "none".into(),
            AutoSizeMode::Left => "left".into(),
            AutoSizeMode::Center => "center".into(),
            AutoSizeMode::Right => "right".into(),
        });
    }

    Ok(Value::Undefined)
}

pub fn set_autosize<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let value = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?;
        this.set_autosize(
            match &*value {
                "left" => AutoSizeMode::Left,
                "center" => AutoSizeMode::Center,
                "right" => AutoSizeMode::Right,
                _ => AutoSizeMode::None,
            },
            &mut activation.context,
        );
    }

    Ok(Value::Undefined)
}

pub fn background_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok((this.background_color()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_background_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        this.set_background_color(activation.context.gc_context, new_color);
    }

    Ok(Value::Undefined)
}

pub fn border<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.has_border().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let border = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();
        this.set_has_border(activation.context.gc_context, border);
    }

    Ok(Value::Undefined)
}

pub fn border_color<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.border_color().into());
    }

    Ok(Value::Undefined)
}

pub fn set_border_color<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let border_color = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_u32(activation)?;
        this.set_border_color(activation.context.gc_context, border_color);
    }

    Ok(Value::Undefined)
}

pub fn default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.new_text_format().as_avm2_object(activation)?.into());
    }

    Ok(Value::Undefined)
}

pub fn set_default_text_format<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let new_text_format = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_object(activation)?;
        let new_text_format = TextFormat::from_avm2_object(new_text_format, activation)?;

        this.set_new_text_format(new_text_format, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn display_as_password<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.is_password().into());
    }

    Ok(Value::Undefined)
}

pub fn set_display_as_password<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_password = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_password(is_password, &mut activation.context);
    }

    Ok(Value::Undefined)
}

pub fn embed_fonts<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok((!this.is_device_font()).into());
    }

    Ok(Value::Undefined)
}

pub fn set_embed_fonts<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let is_embed_fonts = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_boolean();

        this.set_is_device_font(&mut activation.context, !is_embed_fonts);
    }

    Ok(Value::Undefined)
}

pub fn html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(AvmString::new(
            activation.context.gc_context,
            this.html_text(&mut activation.context)?,
        )
        .into());
    }

    Ok(Value::Undefined)
}

pub fn set_html_text<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        let html_text = args
            .get(0)
            .cloned()
            .unwrap_or(Value::Undefined)
            .coerce_to_string(activation)?
            .to_string();

        this.set_html_text(html_text, &mut activation.context)?;
    }

    Ok(Value::Undefined)
}

pub fn length<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this
        .and_then(|this| this.as_display_object())
        .and_then(|this| this.as_edit_text())
    {
        return Ok(this.text_length().into());
    }

    Ok(Value::Undefined)
}

/// Construct `TextField`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.text"), "TextField"),
        Some(QName::new(Namespace::package("flash.display"), "InteractiveObject").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);

    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "autoSize"),
        Method::from_builtin(autosize),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "autoSize"),
        Method::from_builtin(set_autosize),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "backgroundColor"),
        Method::from_builtin(background_color),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "backgroundColor"),
        Method::from_builtin(set_background_color),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "border"),
        Method::from_builtin(border),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "border"),
        Method::from_builtin(set_border),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "borderColor"),
        Method::from_builtin(border_color),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "borderColor"),
        Method::from_builtin(set_border_color),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "defaultTextFormat"),
        Method::from_builtin(default_text_format),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "defaultTextFormat"),
        Method::from_builtin(set_default_text_format),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "displayAsPassword"),
        Method::from_builtin(display_as_password),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "displayAsPassword"),
        Method::from_builtin(set_display_as_password),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "embedFonts"),
        Method::from_builtin(embed_fonts),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "embedFonts"),
        Method::from_builtin(set_embed_fonts),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "htmlText"),
        Method::from_builtin(html_text),
    ));
    write.define_instance_trait(Trait::from_setter(
        QName::new(Namespace::public(), "htmlText"),
        Method::from_builtin(set_html_text),
    ));
    write.define_instance_trait(Trait::from_getter(
        QName::new(Namespace::public(), "length"),
        Method::from_builtin(length),
    ));

    class
}
