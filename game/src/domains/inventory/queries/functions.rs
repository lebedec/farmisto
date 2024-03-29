use crate::inventory::{Function, Installation, InventoryError, Nozzle};

pub type Constructor<T> = fn(usize) -> T;

pub trait FunctionsQuery {
    fn as_tether(&self) -> Result<(), InventoryError>;
    fn as_food(&self) -> Result<(), InventoryError>;
    fn as_seeds<T>(&self, constructor: Constructor<T>) -> Result<T, InventoryError>;
    fn as_hammer(&self) -> Result<(), InventoryError>;
    fn as_shovel(&self) -> Result<(), InventoryError>;
    fn as_stone(&self) -> Result<(), InventoryError>;
    fn as_material(&self) -> Result<u8, InventoryError>;
    fn as_product(&self) -> Result<usize, InventoryError>;
    fn as_installation(&self) -> Result<Installation, InventoryError>;
    fn as_assembly<T>(&self, constructor: Constructor<T>) -> Result<T, InventoryError>;
    fn as_moistener(&self) -> Result<Nozzle, InventoryError>;
    fn as_fertilizer(&self) -> Result<f32, InventoryError>;
}

impl FunctionsQuery for Vec<Function> {
    fn as_tether(&self) -> Result<(), InventoryError> {
        for function in self {
            if let Function::Tether = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_food(&self) -> Result<(), InventoryError> {
        for function in self {
            if let Function::Food = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_seeds<T>(&self, constructor: fn(usize) -> T) -> Result<T, InventoryError> {
        for function in self {
            if let Function::Seeding(kind) = function {
                return Ok(constructor(*kind));
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_hammer(&self) -> Result<(), InventoryError> {
        for function in self {
            if let Function::Instrumenting = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_moistener(&self) -> Result<Nozzle, InventoryError> {
        for function in self {
            if let Function::Moistener(nozzle) = function {
                return Ok(*nozzle);
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_fertilizer(&self) -> Result<f32, InventoryError> {
        for function in self {
            if let Function::Fertilizer(quality) = function {
                return Ok(*quality);
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_shovel(&self) -> Result<(), InventoryError> {
        for function in self {
            if let Function::Shovel = function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_material(&self) -> Result<u8, InventoryError> {
        for function in self {
            if let Function::Material(material) = function {
                return Ok(*material);
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_product(&self) -> Result<usize, InventoryError> {
        for function in self {
            if let Function::Product(kind) = function {
                return Ok(*kind);
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_stone(&self) -> Result<(), InventoryError> {
        for function in self {
            if Function::Stone == *function {
                return Ok(());
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_installation(&self) -> Result<Installation, InventoryError> {
        for function in self {
            if let Function::Installation(installation) = function {
                return Ok(*installation);
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }

    fn as_assembly<T>(&self, constructor: Constructor<T>) -> Result<T, InventoryError> {
        for function in self {
            if let Function::Assembly(kind) = function {
                return Ok(constructor(*kind));
            }
        }
        Err(InventoryError::ItemFunctionNotFound)
    }
}
