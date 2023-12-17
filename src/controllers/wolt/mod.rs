use reqwest;

pub mod types;

pub struct WoltAPI {
    lat: f32,
    lon: f32,
    client: reqwest::Client,
}

impl WoltAPI {
    pub fn new(lat: f32, lon: f32) -> Self {
        let client = reqwest::Client::new();
        WoltAPI { lat, lon, client }
    }

    pub async fn get_all_resteraunts(
        &self,
    ) -> Result<types::GetAllRestaurantsResponse, reqwest::Error> {
        let resp = self
            .client
            .get("https://consumer-api.wolt.com/v1/pages/venue-list/lunch-venues")
            .query(&[("lat", self.lat), ("lon", self.lon)])
            .send()
            .await
            .unwrap()
            .json::<types::GetAllRestaurantsResponse>()
            .await
            .unwrap();

        Ok(resp)
    }
}
