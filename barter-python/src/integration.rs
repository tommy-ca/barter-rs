use std::fmt;

use barter_integration::snapshot::{SnapUpdates, Snapshot};
use pyo3::{
    Bound, Py, PyObject, PyResult, Python,
    exceptions::PyNotImplementedError,
    prelude::*,
    pyclass::CompareOp,
    types::{PyAny, PyType},
};

#[pyclass(module = "barter_python", name = "Snapshot", unsendable)]
pub struct PySnapshot {
    inner: Snapshot<Py<PyAny>>,
}

impl PySnapshot {
    pub(crate) fn from_inner(inner: Snapshot<Py<PyAny>>) -> Self {
        Self { inner }
    }

    pub(crate) fn clone_inner(&self, py: Python<'_>) -> Snapshot<Py<PyAny>> {
        Snapshot::new(self.inner.value().clone_ref(py))
    }

    fn repr_value(&self, py: Python<'_>) -> PyResult<String> {
        let owned = self.inner.value().clone_ref(py);
        owned.bind(py).repr()?.extract()
    }

    fn equals_value(&self, other: &Self, py: Python<'_>) -> PyResult<bool> {
        let lhs = self.inner.value().clone_ref(py);
        let rhs = other.inner.value().clone_ref(py);
        lhs.bind(py)
            .rich_compare(rhs.bind(py), CompareOp::Eq)?
            .extract()
    }
}

#[pymethods]
impl PySnapshot {
    #[new]
    #[pyo3(signature = (value))]
    pub fn __new__(value: PyObject) -> Self {
        Self {
            inner: Snapshot::new(value),
        }
    }

    #[classmethod]
    #[pyo3(signature = (value))]
    pub fn new(_cls: &Bound<'_, PyType>, value: PyObject) -> Self {
        Self {
            inner: Snapshot::new(value),
        }
    }

    #[getter]
    pub fn value(&self, py: Python<'_>) -> PyObject {
        self.inner.value().clone_ref(py).into_py(py)
    }

    pub fn as_ref(&self, py: Python<'_>) -> Self {
        Self {
            inner: Snapshot::new(self.inner.value().clone_ref(py)),
        }
    }

    #[pyo3(signature = (func))]
    pub fn map(&self, py: Python<'_>, func: &Bound<'_, PyAny>) -> PyResult<Self> {
        let current = self.inner.value().clone_ref(py);
        let mapped = func.call1((current,))?;
        Ok(Self {
            inner: Snapshot::new(mapped.into_py(py)),
        })
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| self.repr_value(py))
            .map(|value_repr| format!("Snapshot({value_repr})"))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Python::with_gil(|py| self.equals_value(other, py)),
            CompareOp::Ne => {
                Python::with_gil(|py| self.equals_value(other, py)).map(|result| !result)
            }
            _ => Err(PyNotImplementedError::new_err(
                "ordering comparisons are not supported for Snapshot",
            )),
        }
    }
}

impl<T> From<Snapshot<T>> for PySnapshot
where
    T: IntoPy<PyObject>,
{
    fn from(snapshot: Snapshot<T>) -> Self {
        Python::with_gil(|py| {
            let converted = snapshot.map(|value| value.into_py(py));
            Self { inner: converted }
        })
    }
}

impl fmt::Debug for PySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Python::with_gil(|py| match self.repr_value(py) {
            Ok(value) => write!(f, "PySnapshot({value})"),
            Err(_) => write!(f, "PySnapshot(<unrepr>)"),
        })
    }
}

#[pyclass(module = "barter_python", name = "SnapUpdates", unsendable)]
pub struct PySnapUpdates {
    pub(crate) inner: SnapUpdates<Snapshot<Py<PyAny>>, Py<PyAny>>,
}

impl PySnapUpdates {
    fn repr_parts(&self, py: Python<'_>) -> PyResult<(String, String)> {
        let snapshot_repr = self.inner.snapshot.value().clone_ref(py).bind(py).repr()?;
        let updates_repr = self.inner.updates.clone_ref(py).bind(py).repr()?;
        Ok((snapshot_repr.extract()?, updates_repr.extract()?))
    }

    fn snapshots_equal(&self, other: &Self, py: Python<'_>) -> PyResult<bool> {
        self.inner
            .snapshot
            .value()
            .clone_ref(py)
            .bind(py)
            .rich_compare(
                other.inner.snapshot.value().clone_ref(py).bind(py),
                CompareOp::Eq,
            )?
            .extract()
    }

    fn updates_equal(&self, other: &Self, py: Python<'_>) -> PyResult<bool> {
        self.inner
            .updates
            .clone_ref(py)
            .bind(py)
            .rich_compare(other.inner.updates.clone_ref(py).bind(py), CompareOp::Eq)?
            .extract()
    }
}

#[pymethods]
impl PySnapUpdates {
    #[new]
    #[pyo3(signature = (snapshot, updates))]
    pub fn __new__(py: Python<'_>, snapshot: Py<PySnapshot>, updates: PyObject) -> PyResult<Self> {
        let snapshot_inner = snapshot.borrow(py).clone_inner(py);
        Ok(Self {
            inner: SnapUpdates::new(snapshot_inner, updates),
        })
    }

    #[classmethod]
    #[pyo3(signature = (snapshot, updates))]
    pub fn new(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        snapshot: Py<PySnapshot>,
        updates: PyObject,
    ) -> PyResult<Self> {
        Self::__new__(py, snapshot, updates)
    }

    #[getter]
    pub fn snapshot(&self, py: Python<'_>) -> PySnapshot {
        PySnapshot::from_inner(self.inner.snapshot.clone_inner(py))
    }

    #[getter]
    pub fn updates(&self, py: Python<'_>) -> PyObject {
        self.inner.updates.clone_ref(py).into_py(py)
    }

    fn __repr__(&self) -> PyResult<String> {
        Python::with_gil(|py| self.repr_parts(py)).map(|(snapshot, updates)| {
            format!("SnapUpdates(snapshot={snapshot}, updates={updates})")
        })
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Eq => Python::with_gil(|py| {
                if !self.snapshots_equal(other, py)? {
                    return Ok(false);
                }
                self.updates_equal(other, py)
            }),
            CompareOp::Ne => self.__richcmp__(other, CompareOp::Eq).map(|result| !result),
            _ => Err(PyNotImplementedError::new_err(
                "ordering comparisons are not supported for SnapUpdates",
            )),
        }
    }
}

impl fmt::Debug for PySnapUpdates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Python::with_gil(|py| match self.repr_parts(py) {
            Ok((snapshot, updates)) => {
                write!(f, "PySnapUpdates(snapshot={snapshot}, updates={updates})")
            }
            Err(_) => write!(f, "PySnapUpdates(<unrepr>)"),
        })
    }
}

impl<U, V> From<SnapUpdates<Snapshot<U>, V>> for PySnapUpdates
where
    U: IntoPy<PyObject>,
    V: IntoPy<PyObject>,
{
    fn from(value: SnapUpdates<Snapshot<U>, V>) -> Self {
        Python::with_gil(|py| {
            let SnapUpdates { snapshot, updates } = value;
            let converted_snapshot = snapshot.map(|inner| inner.into_py(py));
            let converted_updates = updates.into_py(py);
            Self {
                inner: SnapUpdates::new(converted_snapshot, converted_updates),
            }
        })
    }
}

trait SnapshotCloneExt {
    fn clone_inner(&self, py: Python<'_>) -> Snapshot<Py<PyAny>>;
}

impl SnapshotCloneExt for Snapshot<Py<PyAny>> {
    fn clone_inner(&self, py: Python<'_>) -> Snapshot<Py<PyAny>> {
        Snapshot::new(self.value().clone_ref(py))
    }
}
