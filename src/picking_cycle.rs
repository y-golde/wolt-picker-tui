use crate::app::App;
use crate::controllers;
use crate::controllers::WoltAPITypes::ResterauntItem;
use controllers::WoltAPITypes::GetAllRestaurantsResponse;
use rand::Rng;
pub struct PickingCycle {
    address: (f32, f32), // impl!
    liked_category: String,
    disliked_categories: Vec<String>,
    restaurants: Option<GetAllRestaurantsResponse>,
    app_instance: App,
}

impl PickingCycle {
    pub fn new() -> Self {
        let address = PickingCycle::get_addr();
        let app_instance = App::new(address);

        PickingCycle {
            address,
            liked_category: String::from(""),
            disliked_categories: vec![],
            restaurants: None,
            app_instance,
        }
    }

    fn get_addr() -> (f32, f32) {
        // TODO: impl with addr
        return (32.08462100522144, 34.8215676471591);
    }

    /*
     * lazily getting the resteraunt list
     */
    async fn get_restaurants(&mut self) -> &GetAllRestaurantsResponse {
        match &self.restaurants {
            None => {
                let (lat, lon) = self.address;
                let api = controllers::WoltAPI::new(lat, lon);
                let resteraunts = api.get_all_resteraunts().await.unwrap();
                let resteraunts_option = Some(resteraunts);
                self.restaurants = resteraunts_option;

                &self.restaurants.as_ref().unwrap()
            }
            Some(_) => self.restaurants.as_ref().unwrap(),
        }
    }

    async fn get_random_restaurant_pool(&mut self) -> ResterauntItem {
        let resteraunts_clone = self.get_restaurants().await.clone();
        let restaurants_items = &resteraunts_clone.sections[0].items;

        let mut matching_items = vec![];

        for item in restaurants_items {
            let mut is_liked = self.liked_category == ""; // if there are no liked category set - automatically flag it as true
            let mut is_disliked = false;
            for category in item.filtering.filters[0].values.clone() {
                if category == self.liked_category {
                    is_liked = true
                }

                // todo: do something smarter than cloning :P
                for disliked_category in self.disliked_categories.clone() {
                    if category == disliked_category {
                        is_disliked = true;
                    }
                }
            }

            if is_liked && !is_disliked {
                matching_items.push(item);
            }
        }

        let rand_index = rand::thread_rng().gen_range(0..matching_items.len());

        matching_items[rand_index].clone()
    }

    pub async fn start(&mut self) {
        let random_restaurant = self.get_random_restaurant_pool().await;
        self.app_instance
            .display_restaurant(&random_restaurant)
            .unwrap();
    }

    // move to tui funstuff
}
