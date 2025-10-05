use std::fmt;

use barter_integration::collection::{
    none_one_or_many::NoneOneOrMany,
    one_or_many::OneOrMany,
};
use pyo3::{
    Bound, Py, Python,
    exceptions::PyValueError,
    prelude::*,
    pyclass::CompareOp,
    types::{PyAny, PyBytes, PyDict, PyIterator, PyList, PyModule, PyString, PyTuple, PyType},
};
use serde::Serialize;

#[pyclass(module = "barter_python", name = "NoneOneOrMany", unsendable)]
pub struct PyNoneOneOrMany {
    pub(crate) inner: NoneOneOrMany<Py<PyAny>>,
}

#[pyclass(module = "barter_python", name = "OneOrMany", unsendable)]
pub struct PyOneOrMany {
    pub(crate) inner: OneOrMany<Py<PyAny>>,
}

impl PyNoneOneOrMany {
    pub(crate) fn empty() -> Self {
        Self {
            inner: NoneOneOrMany::None,
        }
    }

    pub(crate) fn from_serializable<T>(py: Python<'_>, value: NoneOneOrMany<T>) -> PyResult<Self>
    where
        T: Serialize,
    {
        match value {
            NoneOneOrMany::None => Ok(Self::empty()),
            NoneOneOrMany::One(item) => {
                let py_item = serialize_to_python(py, item)?;
                Ok(Self {
                    inner: NoneOneOrMany::One(py_item),
                })
            }
            NoneOneOrMany::Many(items) => {
                let mut converted = Vec::with_capacity(items.len());
                for item in items {
                    converted.push(serialize_to_python(py, item)?);
                }
                Ok(Self {
                    inner: NoneOneOrMany::Many(converted),
                })
            }
        }
    }

    fn to_py_list(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::empty_bound(py);
        match &self.inner {
            NoneOneOrMany::None => {}
            NoneOneOrMany::One(item) => list.append(item.clone_ref(py))?,
            NoneOneOrMany::Many(items) => {
                for item in items {
                    list.append(item.clone_ref(py))?;
                }
            }
        }
        Ok(list.into())
    }

    fn variant_name(&self) -> &'static str {
        match &self.inner {
            NoneOneOrMany::None => "None",
            NoneOneOrMany::One(_) => "One",
            NoneOneOrMany::Many(_) => "Many",
        }
    }
}

impl PyOneOrMany {
    fn to_py_list(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::empty_bound(py);
        match &self.inner {
            OneOrMany::One(item) => list.append(item.clone_ref(py))?,
            OneOrMany::Many(items) => {
                for item in items {
                    list.append(item.clone_ref(py))?;
                }
            }
        }
        Ok(list.into())
    }

    fn variant_name(&self) -> &'static str {
        match &self.inner {
            OneOrMany::One(_) => "One",
            OneOrMany::Many(_) => "Many",
        }
    }
}

#[pymethods]
impl PyNoneOneOrMany {
    #[new]
    #[pyo3(signature = (*values))]
    fn __new__(values: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let py = values.py();

        match values.len() {
            0 => Ok(Self::empty()),
            1 => {
                let first = values.get_item(0)?;
                if first.is_none() {
                    return Ok(Self::empty());
                }

                if let Some(items) = extract_iterable(py, &first)? {
                    return Ok(Self {
                        inner: NoneOneOrMany::from(items),
                    });
                }

                Ok(Self {
                    inner: NoneOneOrMany::One(first.into_py(py)),
                })
            }
            _ => {
                let mut items = Vec::with_capacity(values.len());
                for value in values.iter() {
                    items.push(value.into_py(py));
                }
                Ok(Self {
                    inner: NoneOneOrMany::from(items),
                })
            }
        }
    }

    #[classmethod]
    fn none(_cls: &Bound<'_, PyType>) -> Self {
        Self::empty()
    }

    #[classmethod]
    fn one(_cls: &Bound<'_, PyType>, value: PyObject) -> Self {
        Self {
            inner: NoneOneOrMany::One(value.into()),
        }
    }

    #[classmethod]
    fn many(_cls: &Bound<'_, PyType>, iterable: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = iterable.py();
        let items = extract_iterable(py, iterable)?.ok_or_else(|| {
            PyValueError::new_err("expected an iterable to construct NoneOneOrMany.many")
        })?;
        Ok(Self {
            inner: NoneOneOrMany::from(items),
        })
    }

    #[getter]
    fn is_none(&self) -> bool {
        self.inner.is_none()
    }

    #[getter]
    fn is_one(&self) -> bool {
        self.inner.is_one()
    }

    #[getter]
    fn is_many(&self) -> bool {
        self.inner.is_many()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn to_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.to_py_list(py)?.into_py(py))
    }

    fn __iter__(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let list = self.to_py_list(py)?;
            let iterator = list.bind(py).call_method0("__iter__")?;
            Ok(iterator.into_py(py))
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let list = self.to_py_list(py)?;
            let repr: String = list.bind(py).repr()?.extract()?;
            Ok(format!(
                "NoneOneOrMany(kind={}, values={repr})",
                self.variant_name()
            ))
        })
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        Python::with_gil(|py| {
            let lhs = self.to_py_list(py)?;
            let rhs = other.to_py_list(py)?;
            let result = lhs.bind(py).rich_compare(rhs.bind(py), op)?;
            result.extract()
        })
    }
}

#[pymethods]
impl PyOneOrMany {
    #[new]
    #[pyo3(signature = (*values))]
    fn __new__(values: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let py = values.py();
        if values.len() == 0 {
            return Err(PyValueError::new_err(
                "OneOrMany requires at least one value",
            ));
        }

        if values.len() == 1 {
            let first = values.get_item(0)?;

            if let Some(items) = extract_iterable(py, &first)? {
                if items.is_empty() {
                    return Err(PyValueError::new_err(
                        "OneOrMany iterable must contain at least one element",
                    ));
                }
                return Ok(Self {
                    inner: OneOrMany::from(items),
                });
            }

            return Ok(Self {
                inner: OneOrMany::One(first.into_py(py)),
            });
        }

        let mut items = Vec::with_capacity(values.len());
        for value in values.iter() {
            items.push(value.into_py(py));
        }
        Ok(Self {
            inner: OneOrMany::from(items),
        })
    }

    #[classmethod]
    fn one(_cls: &Bound<'_, PyType>, value: PyObject) -> Self {
        Self {
            inner: OneOrMany::One(value.into()),
        }
    }

    #[classmethod]
    fn many(_cls: &Bound<'_, PyType>, iterable: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = iterable.py();
        let items = extract_iterable(py, iterable)?.ok_or_else(|| {
            PyValueError::new_err("expected an iterable to construct OneOrMany.many")
        })?;

        if items.is_empty() {
            return Err(PyValueError::new_err(
                "OneOrMany iterable must contain at least one element",
            ));
        }

        Ok(Self {
            inner: OneOrMany::from(items),
        })
    }

    #[getter]
    fn is_one(&self) -> bool {
        self.inner.is_one()
    }

    #[getter]
    fn is_many(&self) -> bool {
        self.inner.is_many()
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn to_list(&self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.to_py_list(py)?.into_py(py))
    }

    fn __iter__(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            let list = self.to_py_list(py)?;
            let iterator = list.bind(py).call_method0("__iter__")?;
            Ok(iterator.into_py(py))
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| {
            let list = self.to_py_list(py)?;
            let repr: String = list.bind(py).repr()?.extract()?;
            Ok(format!(
                "OneOrMany(kind={}, values={repr})",
                self.variant_name()
            ))
        })
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        Python::with_gil(|py| {
            let lhs = self.to_py_list(py)?;
            let rhs = other.to_py_list(py)?;
            let result = lhs.bind(py).rich_compare(rhs.bind(py), op)?;
            result.extract()
        })
    }
}

pub(crate) fn wrap_none_one_or_many<T>(
    py: Python<'_>,
    value: NoneOneOrMany<T>,
) -> PyResult<Py<PyNoneOneOrMany>>
where
    T: Serialize,
{
    let wrapper = PyNoneOneOrMany::from_serializable(py, value)?;
    Py::new(py, wrapper)
}

fn extract_iterable(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Option<Vec<Py<PyAny>>>> {
    if value.is_instance_of::<PyString>() || value.is_instance_of::<PyBytes>() {
        return Ok(None);
    }

    if value.is_instance_of::<PyDict>() {
        return Ok(None);
    }

    if let Ok(list) = value.downcast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(item.into_py(py));
        }
        return Ok(Some(items));
    }

    if let Ok(tuple) = value.downcast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(item.into_py(py));
        }
        return Ok(Some(items));
    }

    if value.hasattr("__iter__")? {
        let iterator: Bound<'_, PyIterator> = value.iter()?;
        let mut items = Vec::new();
        for item in iterator {
            let element = item?;
            items.push(element.into_py(py));
        }
        return Ok(Some(items));
    }

    Ok(None)
}

fn serialize_to_python<T>(py: Python<'_>, value: T) -> PyResult<Py<PyAny>>
where
    T: Serialize,
{
    let json =
        serde_json::to_string(&value).map_err(|err| PyValueError::new_err(err.to_string()))?;
    let json_module = PyModule::import_bound(py, "json")?;
    let loads = json_module.getattr("loads")?;
    let py_value = loads.call1((json,))?;
    Ok(py_value.into())
}

impl fmt::Debug for PyNoneOneOrMany {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Python::with_gil(|py| match self.to_py_list(py) {
            Ok(list) => match list.bind(py).repr() {
                Ok(repr) => match repr.extract::<String>() {
                    Ok(text) => write!(
                        f,
                        "PyNoneOneOrMany(kind={}, values={text})",
                        self.variant_name()
                    ),
                    Err(_) => write!(f, "PyNoneOneOrMany(<unrepr>)"),
                },
                Err(_) => write!(f, "PyNoneOneOrMany(<repr error>)"),
            },
            Err(_) => write!(f, "PyNoneOneOrMany(<error>)"),
        })
    }
}

impl fmt::Debug for PyOneOrMany {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Python::with_gil(|py| match self.to_py_list(py) {
            Ok(list) => match list.bind(py).repr() {
                Ok(repr) => match repr.extract::<String>() {
                    Ok(text) => write!(
                        f,
                        "PyOneOrMany(kind={}, values={text})",
                        self.variant_name()
                    ),
                    Err(_) => write!(f, "PyOneOrMany(<unrepr>)"),
                },
                Err(_) => write!(f, "PyOneOrMany(<repr error>)"),
            },
            Err(_) => write!(f, "PyOneOrMany(<error>)"),
        })
    }
}
