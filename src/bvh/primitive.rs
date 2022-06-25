// Copyright © 2022
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use crate::*;

pub trait BvhPrimitive {
    fn centroid(&self) -> &Point3;
    fn min(&self) -> Point3;
    fn max(&self) -> Point3;
}
