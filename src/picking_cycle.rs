use crate::app::App;
use crate::controllers;
use crate::controllers::WoltAPITypes::ResterauntItem;
use controllers::WoltAPITypes::GetAllRestaurantsResponse;
use rand::Rng;
pub struct PickingCycle {
    address: String, // impl!
    liked_category: String,
    disliked_categories: Vec<String>,
    restaurants: Option<GetAllRestaurantsResponse>,
    app_instance: App,
}

impl PickingCycle {
    pub fn new() -> Self {
        let app_instance = App::new();
        PickingCycle {
            address: String::from(""),
            liked_category: String::from(""),
            disliked_categories: vec![],
            restaurants: None,
            app_instance,
        }
    }

    fn get_lat_lon(&self) -> (f32, f32) {
        // TODO: impl with addr
        return (32.08462100522144, 34.8215676471591);
    }

    /*
     * lazily getting the resteraunt list
     */
    async fn get_restaurants(&mut self) -> &GetAllRestaurantsResponse {
        match &self.restaurants {
            None => {
                let (lat, lon) = self.get_lat_lon();
                let api = controllers::WoltAPI::new(lat, lon);
                let resteraunts = api.get_all_resteraunts().await.unwrap();
                let resteraunts_option = Some(resteraunts);
                self.restaurants = resteraunts_option;

                &self.restaurants.as_ref().unwrap()
            }
            Some(_) => self.restaurants.as_ref().unwrap(),
        }
    }

    async fn get_random_restaurants(&mut self) -> ResterauntItem {
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
        let pick: ResterauntItem;

        loop {
            let random_restaurant = self.get_random_restaurants().await;
            let categories = random_restaurant.filtering.filters[0].values.clone();

            let top_panel_content = format!(
                "do you want to eat at {title} \n {tags} \n",
                title = random_restaurant.title,
                tags = { format!("{:?}", &categories) },
            );
            let want_to_eat_at = self
                .app_instance
                .prompt_question(
                    top_panel_content.as_str(),
                    &self.liked_category,
                    &self.disliked_categories,
                )
                .unwrap();

            if want_to_eat_at == "yes" {
                pick = random_restaurant;
                break;
            }

            let random_category =
                &categories[rand::thread_rng().gen_range(0..categories.len())].clone();
            let liked_or_disliked = self
                .app_instance
                .prompt_question(
                    &format!("are you in the mood for {category} today? type 'yes' to find another {category} place, 'no' to filter {category} out or anything else to skip it", category = random_category),
                    &self.liked_category,
                    &self.disliked_categories
                ).unwrap();

            if liked_or_disliked == "yes" {
                self.liked_category = random_category.to_string();
            } else if liked_or_disliked == "no" {
                self.disliked_categories.push(random_category.to_string());
            }
        }
        println!(
            "It's a match! {} it is!, visit the website at {}",
            pick.title, pick.link.target
        );
    }

    // move to tui funstuff
}
