mod geolocation;

use serde_json::json;
use thiserror::Error;

use std::marker::PhantomData;
use std::env;

/// These marker traits are supposed to protect the client from invalid api key users
#[derive(Debug)]
pub struct Validated;

#[derive(Debug)]
pub struct Invalidated;

#[derive(Error, Debug)]
pub enum GmapsClientError {
    #[error("Failed to validate API KEY")]
    InvalidApiKey,
    
    #[error("Failed sending the request")]
    RequestFailure,
    
    #[error("Missing API KEY, the GMAPS_API_KEY variable may not be set")]
    MissingApiKey,
}

#[derive(Debug)]
pub struct GMapsClient<T = Invalidated> {
    api_key: String,
    base_url: String,
    state: PhantomData<T>,
}

impl GMapsClient<Invalidated> {

    /// Constructs a GMapsClient object 
    /// Env variable GMAPS_API_KEY must be set to a valid api key otherwise
    /// this function errors out
    /// 
    /// returns: Result<GMapsClient, GmapsCError>
    pub fn new() -> Result<GMapsClient<Invalidated>, GmapsClientError> {

        let api_key = match env::var("GMAPS_API_KEY") {
            Ok(val) => Ok(val),
            Err(_) => Err(GmapsClientError::MissingApiKey),
        }?;

        Ok(GMapsClient {
            api_key: api_key,
            base_url: "https://maps.googleapis.com/".to_string(),
            state: PhantomData
        })
    }

    /// Validates the api key by calling the places api
    /// If valid this function returns GmapsClient<Validated> which gives access
    /// to the api, consuming self in the process 
    /// 
    /// returns: Result<GMapsClient<Validated>, GmapsClientError>
    pub async fn validate_api_key(self) -> Result<GMapsClient<Validated>, GmapsClientError> {
        
        let base_url = "https://maps.googleapis.com/".to_string();

        let url = format!("{}/maps/api/place/findplacefromtext/json?input={}&inputtype=textquery&fields=name,place_id,geometry,formatted_address&locationbias=point:50,10&key={}",
            base_url, "bosfor alba", self.api_key);
    
        let response = 
            reqwest::get(url)
            .await
            .map_err(|_| GmapsClientError::RequestFailure)?
            .json::<serde_json::Value>()
            .await
            .map_err(|_| GmapsClientError::RequestFailure)?;

        if response["status"] == json!("REQUEST_DENIED") {
            return Err(GmapsClientError::InvalidApiKey);
        }
        
        Ok(GMapsClient {
            api_key: self.api_key,
            base_url: self.base_url,
            state: PhantomData,
        })

    }

}

impl GMapsClient<Validated> {
    
    /// Queries the places api obtaining the details of a single place given as text
    /// 
    /// parameters:
    ///     * place: Description of the desired place in natural language
    /// returns: serde_json::Value 
    ///
    pub async fn find_single_place_from_text(&self, place: &str) -> serde_json::Value {    
        
        let url = format!("{}/maps/api/place/findplacefromtext/json?input={}&inputtype=textquery&fields=name,place_id,geometry,formatted_address&locationbias=point:50,10&key={}",
            self.base_url, place, self.api_key);
        
        let response = 
            reqwest::get(url)
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
    
        response
    }

    /// Queries the places api obtaining a list of places and their details given a natural language query
    /// 
    /// parameters:
    ///     * query: Description of the desired place in natural language
    /// returns: serde_json::Value 
    pub async fn find_places_from_text(&self, query: &str) -> serde_json::Value {

        let url = format!(
            "{}/maps/api/place/textsearch/json?query={}&radius={}&key={}",
            self.base_url,
            query,
            5000,
            self.api_key,
            );
        
        let response = reqwest::get(url)
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        
        response
    }
}



#[cfg(test)]
pub mod tests {

    use super::*;
    use tokio;

    #[tokio::test]
    pub async fn test_valid_single_place() {

        let gmaps = GMapsClient::new().unwrap();
        let gmaps = gmaps.validate_api_key().await.unwrap();

        let response = gmaps.find_places_from_text("pizza party alba iulia").await;
        let results = response["results"].clone();

        println!("{}", results);
    }

}
