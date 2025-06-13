use thiserror::Error;
use miette::{self, Diagnostic};

type ErrorSpan = (usize, usize);

#[derive(Debug, Error, Diagnostic, Clone)]
pub enum EvalError {
    #[error("Cannot redefine a reserved word or built-in identifier")]
    CannotRedefineBuiltin {
        #[label("Reserved word or built-in identifier")]
        ident: ErrorSpan,
    },

    #[error("Can only call `embed(<expr1>,<expr2>)` if <expr1>: Point1 and <expr2>: Point2")]
    EmbedHasIncorrectTypes {
        #[label("{type1}")]
        span1: ErrorSpan,

        #[label("{type2}")]
        span2: ErrorSpan,

        type1: String,
        type2: String,

    },

    #[error("Compressed point is not on curve {curve}")]
    PointNotOnCurve {
        #[label("{pt_type}")]
        span: ErrorSpan,
        pt_type: String,
        curve: String,
    },

    #[error("Compressed point is not in group {group}")]
    PointNotInGroup {
        #[label("{pt_type}")]
        span: ErrorSpan,
        pt_type: String,
        group: String,
    },

    #[error("{point_type} can only contain {expected_coord_type} values")]
    PointCanOnlyContainCoord {
        #[label("{x_coord_type}")]
        x_span: ErrorSpan,
        #[label("{y_coord_type}")]
        y_span: ErrorSpan,
        point_type: String,
        expected_coord_type: String,
        x_coord_type: String,
        y_coord_type: String,
    },

    #[error("Coordinate pairs can only contain Coord1 or Coord6 values")]
    PairCanOnlyContainCoord1Or6 {
        #[label("{x_coord_type}")]
        x_span: ErrorSpan,
        #[label("{y_coord_type}")]
        y_span: ErrorSpan,
        x_coord_type: String,
        y_coord_type: String,
    },

    #[error("Coordinate triples can only contain Coord2 values")]
    TripleCanOnlyContainCoord2 {
        #[label("{x_coord_type}")]
        x_span: ErrorSpan,
        #[label("{y_coord_type}")]
        y_span: ErrorSpan,
        #[label("{z_coord_type}")]
        z_span: ErrorSpan,
        x_coord_type: String,
        y_coord_type: String,
        z_coord_type: String,
    },

    #[error("Invalid compressed point")]
    InvalidCompressedPoint {
        #[label("{pt_type}")]
        span: ErrorSpan,
        pt_type: String,
    },

    #[error("No square root for this {expr_type} value")]
    NotASquare {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("`inv` expects a Coord12 operand")]
    InvExpectsCoord12 {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("`sqr` expects a Coord12 operand")]
    SqrExpectsCoord12 {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("`sqrt` expects a Coord1 or Coord2 operand")]
    SqrtExpectsCoords {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("These two expressions can't be multiplied")]
    TypeMismatchMul {
        #[label("{lhs_type}")]
        lhs_span: ErrorSpan,

        #[label("{rhs_type}")]
        rhs_span: ErrorSpan,

        lhs_type: String,
        rhs_type: String,
    },

    #[error("map_to_g1 can only be applied to Coord1 types")]
    TypeMismatchMapToG1 {
        #[label("{type1}")]
        span1: ErrorSpan,
        #[label("{type2}")]
        span2: ErrorSpan,
        type1: String,
        type2: String,
    },

    #[error("map_to_g2 can only be applied to Coord2 types")]
    TypeMismatchMapToG2 {
        #[label("{type1}")]
        span1: ErrorSpan,
        #[label("{type2}")]
        span2: ErrorSpan,
        type1: String,
        type2: String,
    },

    #[error("Can only add expressions of the same type (and type is Point1 or Point2)")]
    TypeMismatchPointAdd {

        #[label("{lhs_type}")]
        lhs_span: ErrorSpan,

        #[label("{rhs_type}")]
        rhs_span: ErrorSpan,

        lhs_type: String,
        rhs_type: String,
    },

    #[error("Can only negate Point1, Point2, Coord1, Coord2")]
    TypeMismatchNeg {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String
    },

    #[error("Can only subtract points of the same type (and type is Point1 or Point2)")]
    TypeMismatchPointSub {
        #[label("{lhs_type}")]
        lhs_span: ErrorSpan,

        #[label("{rhs_type}")]
        rhs_span: ErrorSpan,

        lhs_type: String,
        rhs_type: String,
    },

    #[error("Unknown identifier. Not built-in or bound with `let`")]
    UnknownIdent {
        #[label("unknown")]
        span: ErrorSpan
    },

    #[error("Equality is not supported at all yet, but definitely not on unequal types")]
    EqualityOnSameTypesOnly {
        #[label("{lhs_type}")]
        lhs_span: ErrorSpan,
        #[label("{rhs_type}")]
        rhs_span: ErrorSpan,
        lhs_type: String,
        rhs_type: String,
    },

    #[error("Lists must contain elements that are all of the same type")]
    ListElemsMustBeAllOfSameType {
        #[label("{first_type}")]
        first_span: ErrorSpan,
        #[label("{wrong_type}")]
        wrong_span: ErrorSpan,
        first_type: String,
        wrong_type: String,
    },

    #[error("miller_loop must contain a list of (Point1, Point2) pairs")]
    MillerLoopMustContainListOfMillerPairs {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("Component selectors .x/.y/.z can only be used on point and coordinate expressions")]
    ComponentsOnlyValidForPointsAndCoords {
        #[label("Component selector")]
        component_span: ErrorSpan,
        #[label("{expr_type}")]
        expr_span: ErrorSpan,
        expr_type: String,
    },
    #[error("The component selector .z can only be used on the Coord6 type")]
    ZComponentSelectorOnlyOnCoord6 {
        #[label("Component selector")]
        component_span: ErrorSpan,
        #[label("{expr_type}")]
        expr_span: ErrorSpan,
        expr_type: String,
    },
    #[error("conj can only be used on Point2")]
    ConjugateOnlyOnPoint2 {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("pow cannot have \"base\" of this type")]
    PowCannotBeUsedOnThisBaseType {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },

    #[error("pow must be used with scalar exponents")]
    PowMustHaveScalarExponent {
        #[label("{expr_type}")]
        span: ErrorSpan,
        expr_type: String,
    },



}