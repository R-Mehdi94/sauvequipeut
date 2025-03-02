/// Représente une erreur personnalisée pour la gestion des erreurs dans l'application.
///
/// Cette énumération permet d'unifier plusieurs types d'erreurs (`std::io::Error`, `serde_json::Error`
/// et des erreurs génériques sous forme de `String`).
///
/// # Variantes
/// - `Io(std::io::Error)`: Erreur liée aux opérations d'entrée/sortie (ex: lecture/écriture de fichiers, connexions réseau).
/// - `Json(serde_json::Error)`: Erreur liée à la sérialisation/désérialisation JSON.
/// - `Other(String)`: Une erreur générique sous forme de chaîne de caractères.

#[derive(Debug)]
pub enum MyError {
    /// Erreur d'entrée/sortie (`std::io::Error`).
    Io(std::io::Error),

    /// Erreur de sérialisation/désérialisation JSON (`serde_json::Error`).
    Json(serde_json::Error),

    /// Autre type d'erreur représenté sous forme de texte.
    Other(String),
}

/// Permet de convertir un `String` en `MyError::Other`.
///
/// # Exemple
/// ```
///
/// use common::utils::my_error::MyError;
/// let error: MyError = "Une erreur personnalisée".to_string().into();
/// ```
impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError::Other(err)
    }
}

/// Permet de convertir un `std::io::Error` en `MyError::Io`.
///
/// # Exemple
/// ```
/// use std::fs::File;
/// use common::utils::my_error::MyError;
///
/// let result: Result<File, MyError> = File::open("non_existant.txt").map_err(MyError::from);
/// ```
impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

/// Permet de convertir un `serde_json::Error` en `MyError::Json`.
///
impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> Self {
        MyError::Json(err)
    }
}
