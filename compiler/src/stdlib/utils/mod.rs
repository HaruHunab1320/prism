use std::sync::Arc;
use parking_lot::RwLock;
use std::time::Duration;
use crate::error::Result;
use crate::module::Module;
use crate::value::{Value, ValueKind};

pub fn init_utils_module() -> Result<Arc<RwLock<Module>>> {
    let module = Arc::new(RwLock::new(Module::new("utils".to_string())));

    // sleep function
    let sleep_fn = Value::new(ValueKind::NativeFunction {
        name: "sleep".to_string(),
        arity: 1,
        handler: Arc::new(|args| {
            if let Some(arg) = args.first() {
                match &arg.kind {
                    ValueKind::Number(seconds) => {
                        let duration = Duration::from_secs_f64(*seconds);
                        std::thread::sleep(duration);
                        Ok(Value::new(ValueKind::Nil))
                    }
                    _ => Ok(Value::new(ValueKind::Nil)),
                }
            } else {
                Ok(Value::new(ValueKind::Nil))
            }
        }),
    });

    {
        let mut module = module.write();
        module.export("sleep".to_string(), sleep_fn)?;
    }

    Ok(module)
}
