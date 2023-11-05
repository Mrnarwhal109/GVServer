use actix_web::http::header::LOCATION;
use actix_web::HttpResponse;

// Return an opaque 500 while preserving the error root's cause for logging.
pub fn e500<T>(e: T) -> actix_web::Error
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

// Return a 400 with the user-representation of the validation error as body.
// The error root cause is preserved for logging purposes.
pub fn e400<T: std::fmt::Debug + std::fmt::Display>(e: T) -> actix_web::Error
    where
        T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorBadRequest(e)
}

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}

/// Gives back the unwrapped value of a `Result` if the `Result` is `Ok`.
/// Returns from the calling function if the `Result` is `Err`.
#[macro_export]
macro_rules! ok_or_return_with {
    ( $e:expr, $r:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => return $r,
        }
    }
}

/// Gives back the unwrapped value of an `Option` if the `Option` is `Some`.
/// Returns from the calling function if the `Option` is `None`.
#[macro_export]
macro_rules! some_or_return_with {
    ( $e:expr, $r:expr ) => {
        match $e {
            Some(x) => x,
            None => return $r,
        }
    }
}

pub fn options_eq<T>(a: &Option<T>, b: &Option<T>) -> bool
where T: Eq
{
    if a.is_some() != b.is_some() {
        return false;
    }
    match a {
        Some(x) => {
            b.as_ref() == Some(x)
        },
        None => {
            b.is_none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::options_eq;

    #[test]
    fn none_with_none() {
        let a: Option::<i32> = Option::None;
        let b: Option::<i32> = Option::None;
        assert!(options_eq(&a, &b))
    }

    #[test]
    fn none_with_some() {
        let a: Option::<i32> = Option::None;
        let inner = 222;
        let b: Option::<i32> = Some(inner);
        assert!(!options_eq(&a, &b));

        let a: Option::<Vec<u8>> = Option::None;
        let inner = vec![3u8, 0u8, 1u8, 22u8, 4u8];
        let b: Option::<Vec<u8>> = Some(inner);
        assert!(!options_eq(&a, &b))
    }

    #[test]
    fn some_with_none() {
        let inner = String::from("Fun_Times_Here");
        let a: Option::<String> = Some(inner);
        let b: Option::<String> = Option::None;
        assert!(!options_eq(&a, &b))
    }

    #[test]
    fn some_strings() {
        let inner_a = String::from("Fun_Times_Here");
        let inner_b = String::from("Fun_Times_Here");
        let a: Option::<String> = Some(inner_a);
        let b: Option::<String> = Some(inner_b);
        assert!(options_eq(&a, &b))
    }

    #[test]
    fn some_strs() {
        let inner_a = String::from("Fun_Times_Here");
        let inner_b = String::from("Fun_Times_Here");
        let a_ref = &inner_a;
        let b_ref = &inner_b;
        let a: Option::<&str> = Some(a_ref);
        let b: Option::<&str> = Some(b_ref);
        assert!(options_eq(&a, &b))
    }

    #[test]
    fn some_vecs_bytes() {
        let inner_a = vec![3u8, 0u8, 1u8, 22u8];
        let inner_b = vec![3u8, 0u8, 1u8, 22u8];
        let a: Option::<Vec<u8>> = Some(inner_a);
        let b: Option::<Vec<u8>> = Some(inner_b);
        assert!(options_eq(&a, &b));

        let inner_a = vec![3u8, 0u8, 1u8, 22u8, 4u8];
        let inner_b = vec![3u8, 0u8, 1u8, 22u8];
        let a: Option::<Vec<u8>> = Some(inner_a);
        let b: Option::<Vec<u8>> = Some(inner_b);
        assert!(!options_eq(&a, &b));

        let inner_a = vec![3u8, 0u8, 1u8, 22u8, 4u8];
        let inner_b = vec![3u8, 0u8, 1u8, 22u8];
        let a: Option::<Vec<u8>> = Some(inner_a);
        let b: Option::<Vec<u8>> = Some(inner_b);
        assert!(!options_eq(&a, &b));
    }
}