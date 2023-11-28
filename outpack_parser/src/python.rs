use crate::ParseError;
use crate::query_types as ast;
use pyo3::prelude::*;
use pyo3::types::PyNone;
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn parse_query<'a>(input: &'a str) -> Result<ast::QueryNode<'a>, ParseError> {
    crate::parse_query(input)
}


#[pymodule]
fn outpack_parser(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_query, m)?)?;
    m.add_class::<Latest>()?;
    m.add_class::<Single>()?;
    m.add_class::<Test>()?;
    m.add_class::<Negation>()?;
    m.add_class::<Brackets>()?;
    m.add_class::<BooleanOperator>()?;
    m.add_class::<Literal>()?;
    m.add_class::<LookupThis>()?;
    m.add_class::<LookupParameter>()?;
    m.add_class::<LookupId>()?;
    m.add_class::<LookupName>()?;
    Ok(())
}

impl From<ParseError> for PyErr {
    fn from(err: ParseError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

// parse_query uses this for automatic return type conversion.
// https://github.com/PyO3/pyo3/issues/1595
impl IntoPy<PyObject> for ast::QueryNode<'_> {
    fn into_py(self, py: Python) -> PyObject {
        ToPyObject::to_object(&self, py)
    }
}

impl ToPyObject for ast::QueryNode<'_>  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::QueryNode::Latest(None) => Latest {
                inner: PyNone::get(py).to_object(py),
            }.into_py(py),

            ast::QueryNode::Latest(Some(inner)) => Latest {
                inner: inner.to_object(py),
            }.into_py(py),

            ast::QueryNode::Single(inner) => Single {
                inner: inner.to_object(py),
            }.into_py(py),

            ast::QueryNode::Negation(inner) => Negation {
                inner: inner.to_object(py),
            }.into_py(py),

            ast::QueryNode::Brackets(inner) => Brackets {
                inner: inner.to_object(py),
            }.into_py(py),

            ast::QueryNode::Test { operator, lhs, rhs } => Test {
                operator: operator.to_object(py),
                lhs: lhs.to_object(py),
                rhs: rhs.to_object(py),
            }.into_py(py),

            ast::QueryNode::BooleanOperator { operator, lhs, rhs } => BooleanOperator {
                operator: operator.to_object(py),
                lhs: lhs.to_object(py),
                rhs: rhs.to_object(py),
            }.into_py(py),
        }
    }
}

impl ToPyObject for ast::TestValue<'_>  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::TestValue::Lookup(inner) => inner.to_object(py),
            ast::TestValue::Literal(inner) => inner.to_object(py),
        }
    }
}

impl ToPyObject for ast::Literal<'_>  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::Literal::Bool(b) => Literal {
                value: b.to_object(py),
            },
            ast::Literal::String(s) => Literal {
                value: s.to_object(py),
            },
            ast::Literal::Number(x) => Literal {
                value: x.to_object(py),
            },
        }.into_py(py)
    }
}

impl ToPyObject for ast::Lookup<'_>  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::Lookup::Packet(ast::PacketLookup::Name) => LookupName.into_py(py),
            ast::Lookup::Packet(ast::PacketLookup::Id) => LookupId.into_py(py),
            ast::Lookup::Packet(ast::PacketLookup::Parameter(name)) => LookupParameter {
                name: name.to_string(),
            }.into_py(py),
            ast::Lookup::This(name) => LookupThis {
                name: name.to_string(),
            }.into_py(py),
            ast::Lookup::Environment(name) => LookupEnvironment {
                name: name.to_string(),
            }.into_py(py),
        }
    }
}

impl ToPyObject for ast::Test  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::Test::Equal => Operator::Equal,
            ast::Test::NotEqual => Operator::NotEqual,
            ast::Test::LessThan => Operator::LessThan,
            ast::Test::LessThanOrEqual => Operator::LessThanOrEqual,
            ast::Test::GreaterThan => Operator::GreaterThan,
            ast::Test::GreaterThanOrEqual => Operator::GreaterThanOrEqual,
        }.into_py(py)
    }
}

impl ToPyObject for ast::Operator  {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            ast::Operator::And => Operator::And,
            ast::Operator::Or => Operator::Or,
        }.into_py(py)
    }
}

#[pyclass(get_all)]
struct Test {
    operator: PyObject,
    lhs: PyObject,
    rhs: PyObject,
}

#[pyclass(get_all)]
struct BooleanOperator {
    operator: PyObject,
    lhs: PyObject,
    rhs: PyObject,
}


#[pyclass(get_all)]
struct Latest {
    inner: PyObject,
}

#[pyclass(get_all)]
struct Single {
    inner: PyObject,
}
#[pyclass(get_all)]
struct Negation {
    inner: PyObject,
}

#[pyclass(get_all)]
struct Brackets {
    inner: PyObject,
}

#[pyclass(get_all)]
struct Literal {
    value: PyObject,
}

#[pyclass(get_all)]
struct LookupThis {
    name: String,
}

#[pyclass(get_all)]
struct LookupEnvironment {
    name: String,
}

#[pyclass(get_all)]
struct LookupParameter {
    name: String,
}

#[pyclass]
struct LookupName;

#[pyclass]
struct LookupId;

#[pyclass]
enum Operator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[pymethods]
impl Test {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Test(operator={}, lhs={}, rhs={})",
            self.operator.as_ref(py).repr()?,
            self.lhs.as_ref(py).repr()?,
            self.rhs.as_ref(py).repr()?))
    }
}

#[pymethods]
impl BooleanOperator {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("BooleanOperator(operator={}, lhs={}, rhs={})",
            self.operator.as_ref(py).repr()?,
            self.lhs.as_ref(py).repr()?,
            self.rhs.as_ref(py).repr()?))
    }
}

#[pymethods]
impl Latest {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Latest(inner={})", self.inner.as_ref(py).repr()?))
    }
}

#[pymethods]
impl Single {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Single(inner={})", self.inner.as_ref(py).repr()?))
    }
}

#[pymethods]
impl Negation {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Negation(inner={})", self.inner.as_ref(py).repr()?))
    }
}

#[pymethods]
impl Brackets {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Brackets(inner={})", self.inner.as_ref(py).repr()?))
    }
}

#[pymethods]
impl Literal {
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!("Literal(value={})", self.value.as_ref(py).repr()?))
    }
}

#[pymethods]
impl LookupThis {
    fn __repr__(&self) -> String {
        format!("LookupThis({:?})", self.name)
    }
}

#[pymethods]
impl LookupEnvironment {
    fn __repr__(&self) -> String {
        format!("LookupEnvironment({:?})", self.name)
    }
}

#[pymethods]
impl LookupParameter {
    fn __repr__(&self) -> String {
        format!("LookupParameter({:?})", self.name)
    }
}

#[pymethods]
impl LookupId {
    fn __repr__(&self) -> &'static str {
        "LookupId()"
    }
}

#[pymethods]
impl LookupName {
    fn __repr__(&self) -> &'static str {
        "LookupName()"
    }
}

