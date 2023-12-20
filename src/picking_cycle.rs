use crate::app::{EndScreenApp, RestaurantDisplayApp};
use crate::controllers;
use crate::controllers::WoltAPITypes::ResterauntItem;
use controllers::WoltAPITypes::GetAllRestaurantsResponse;
use rand::prelude::SliceRandom;
use rand::Rng;

pub struct PickingCycle {
    address: (f32, f32), // impl!
    liked_category: String,
    disliked_categories: Vec<String>,
    restaurants: Option<GetAllRestaurantsResponse>,
    restaurant_display_instance: RestaurantDisplayApp,
    end_screen_instance: EndScreenApp,
}

impl PickingCycle {
    pub fn new() -> Self {
        let address = PickingCycle::get_addr();
        let restaurant_display_instance = RestaurantDisplayApp::new(address);
        let end_screen_instance = EndScreenApp::new();

        PickingCycle {
            address,
            liked_category: String::from(""),
            disliked_categories: vec![],
            restaurants: None,
            restaurant_display_instance,
            end_screen_instance,
        }
    }

    fn get_addr() -> (f32, f32) {
        // TODO: impl with addr
        return (32.079612, 34.811399);
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
        let choice: ResterauntItem;

        loop {
            let random_restaurant = self.get_random_restaurant_pool().await;
            let first_question_choices = vec![String::from("yes"), String::from("no")];
            let first_choice_index = self
                .restaurant_display_instance
                .display_restaurant_question(
                    "Do you want to eat at",
                    &random_restaurant,
                    first_question_choices,
                )
                .unwrap();

            if first_choice_index == 0 {
                choice = random_restaurant;
                break;
            }

            let restaurant_categories = random_restaurant.filtering.filters[0].values.to_owned();
            let random_category = restaurant_categories
                .choose(&mut rand::thread_rng())
                .unwrap();

            let second_question_choices = vec![
                String::from("yes"),
                String::from("no"),
                String::from("skip"),
            ];
            let second_choice_index = self
                .restaurant_display_instance
                .display_category_question(
                    format!("are you in the mood for {} today?", random_category).as_str(),
                    &random_restaurant,
                    second_question_choices,
                )
                .unwrap();

            if second_choice_index == 0 {
                self.liked_category = random_category.to_string();
            } else if second_choice_index == 1 {
                self.disliked_categories.push(random_category.to_string());
            }
        }
        let restaurant_name = choice.title;

        let restaurant_slug = choice.venue.slug;

        self.restaurant_display_instance._teardown().unwrap();

        let end_message = format!(
            "{} it is!, go visit https://wolt.com/en/isr/tel-aviv/restaurant/{} to order!",
            restaurant_name, restaurant_slug
        );

        self.end_screen_instance.display_end_screen(end_message);
    }
}
