use common::utils::my_error::MyError;
use std::{net::TcpStream, thread, time};

/// Tente de se connecter à un serveur via une adresse IP et un port.
///
/// Cette fonction effectue une tentative de connexion à un serveur via **TCP**.
/// En cas d'échec, elle affiche une erreur et réessaie toutes les **2 secondes** indéfiniment.
///
/// # Paramètres
/// - `addr`: L'adresse IP ou le nom de domaine du serveur.
/// - `port`: Le port de connexion.
///
/// # Retourne
/// - `Ok(TcpStream)` si la connexion réussit.
/// - `Err(MyError)` si une erreur survient avant d'établir la connexion.
///
/// # Exemple
/// ```no_run
/// use ma_lib::connect_to_server;
///
/// match connect_to_server("127.0.0.1", "8080") {
///     Ok(stream) => println!("Connexion réussie : {:?}", stream),
///     Err(err) => eprintln!("Échec de la connexion : {:?}", err),
/// }
/// ```



pub fn connect_to_server(addr: &str, port: &str) -> Result<TcpStream, MyError> {
    let full_addr = format!("{}:{}", addr, port);

    loop {
        match TcpStream::connect(&full_addr) {
            Ok(stream) => return Ok(stream),
            Err(e) => eprintln!("Erreur de connexion : {}. Nouvelle tentative...", e),
        }

        // Attendre 2 secondes avant une nouvelle tentative
        thread::sleep(time::Duration::from_secs(2));
    }
}
