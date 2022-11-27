use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, HashSet, BTreeMap,
    },
    rc::Rc, hash::Hash,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Hello world");

    Ok(())
}
struct Car {
    days_with_ticket: HashSet<u32>,
    plate: String,
}

impl PartialEq for Car {
    fn eq(&self, other: &Self) -> bool {
        self.plate == other.plate
    }
}

impl Car {
    pub fn has_ticket_on_day(&self, timestamp: u32) -> bool {
        todo!();
    }
    pub fn give_ticket_for_day(&mut self, timestamp: u32) -> bool {
        todo!();
    }
    pub fn plate(&self) -> &str {
        &self.plate
    }
}

impl From<String> for Car {
    fn from(plate: String) -> Self {
        Self {
            days_with_ticket: Default::default(),
            plate,
        }
    }
}

struct CarRegistry {
    cars: HashMap<String, Rc<Car>>,
}

impl CarRegistry {
    pub fn get_car(&mut self, plate: &str) -> Rc<Car> {
        match self.cars.entry(plate.to_owned()) {
            Occupied(o) => o.get().clone(),
            Vacant(v) => {
                let car_ref = Rc::new(Car::from(v.key().clone()));
                v.insert(car_ref.clone());
                car_ref
            }
        }
    }
}

impl Default for CarRegistry {
    fn default() -> Self {
        Self {
            cars: Default::default(),
        }
    }
}

#[derive(PartialEq)]
struct Ticket {
    car: Rc<Car>,
    road_id: u16,
    mile1: u16,
    timestamp1: u32,
    mile2: u16,
    timestamp2: u32,
    // speed: u16 // Calculated
}

fn speed(timestamp1: u32, mile1: u16, timestamp2: u32, mile2: u16) -> u16 {
    assert!(timestamp2 > timestamp1);
    assert!(mile2 > mile1);

    let time_delta_hours = {
        let seconds_per_hour = 60.0 * 60.0;
        let time_delta = (timestamp2 - timestamp1) as f64;
        time_delta / seconds_per_hour
    };

    let distance_miles = (mile2 - mile1) as f64;

    let speed_average = (distance_miles / time_delta_hours) * 100.0;

    speed_average as u16
}

impl Ticket {
    fn calculate_speed(&self) -> u16 {
        speed(self.timestamp1, self.mile1, self.timestamp2, self.mile2)
    }
}

trait Dispatcher {
    type UniqueId: Eq + Hash;

    fn send_ticket(&self, ticket: Ticket);
    fn id(&self) -> Self::UniqueId;
}


struct RoadMonitor<D: Dispatcher> {
    road_id: u16,
    pending_tickets: Vec<Ticket>,
    dispatchers: HashMap<D::UniqueId, Rc<D>>,

    // Car -> Timestamp -> Location on road
    observations: HashMap<String, BTreeMap<u32, u16>>,
}

impl<D: Dispatcher> From<u16> for RoadMonitor<D> {
    fn from(road_id: u16) -> Self {
        RoadMonitor {
            road_id,
            pending_tickets: Default::default(),
            dispatchers: Default::default(),
            observations: Default::default(),
        }
    }
}

impl<D: Dispatcher> RoadMonitor<D> {
    fn record_observation(&mut self, car: Rc<Car>, mile: u16, time: u32, limit: u16) {
        match self.observations.entry(car.plate.clone()) {
            Occupied(o) => {},
            Vacant(v) => {v.insert(Default::default());},
        };

        let location_map = self.observations.get_mut(car.plate().clone()).expect("map must be present now");

        location_map.insert(time, mile);

        let parwise = location_map.iter().zip(location_map.iter().skip(1));

        for ((prev_time, prev_mile), (next_time, next_mile)) in parwise {
            let speed = speed(*prev_time, *prev_mile, *next_time, *next_mile);
            if speed > 100 * limit {
                if let Some(dispatcher) = self.dispatchers.values().nth(0) {
                    dispatcher.send_ticket(Ticket {
                        car: car.clone(),
                        road_id: self.road_id,
                        mile1: *prev_mile,
                        timestamp1: *prev_time,
                        mile2: *next_mile,
                        timestamp2: *next_time,
                    });
                }
            }
        }
    }

    fn add_ticket_dispatcher(&mut self, d: Rc<D>) {
        match self.dispatchers.entry(d.id()) {
            Occupied(_) => todo!(),
            Vacant(v) => {
                v.insert(d.clone());
            },
        }
    }
    fn remove_ticket_dispatcher(&mut self, d: &D) {
        todo!();
    }
}

struct RoadNetwork<D: Dispatcher> {
    road_monitors: HashMap<u16, Rc<RoadMonitor<D>>>,
}

impl<D: Dispatcher> RoadNetwork<D> {
    fn get_road_monitor(&mut self, road_id: u16) -> Rc<RoadMonitor<D>> {
        todo!();
    }
}
struct Camera<D: Dispatcher> {
    road_monitor: Rc<RoadMonitor<D>>,
    mile: u16,
    speed_limit: u16, // Passed to the RoadMonitor?
}

impl<D: Dispatcher> Camera<D> {
    fn observation(&mut self, car: Rc<Car>, time: u32) {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};

    use crate::{Car, CarRegistry, Dispatcher, RoadMonitor, Ticket};

    #[test]
    fn test_car_registry() {
        let mut registry = CarRegistry::default();

        let car_ref = registry.get_car("UN1X");
        let car_ref2 = registry.get_car("UN1X");

        assert_eq!(car_ref.plate(), car_ref2.plate());
    }

    #[test]
    fn test_ticket_calculation() {
        let t = Ticket {
            car: Rc::new(Car::from("UNIX".to_owned())),
            road_id: 66,
            mile1: 100,
            timestamp1: 123456,
            mile2: 110,
            timestamp2: 123816,
        };

        assert_eq!(t.calculate_speed(), 10000);

        let t = Ticket {
            car: Rc::new(Car::from("RE05BKG".to_owned())),
            road_id: 368,
            mile1: 1234,
            timestamp1: 1000000,
            mile2: 1235,
            timestamp2: 1000060,
        };

        assert_eq!(t.calculate_speed(), 6000);
    }

    struct TestDispatcher {
        id: u32,
        tickets: RefCell<Vec<Ticket>>,
    }

    impl TestDispatcher {
        fn has_ticket(&self, ticket: &Ticket) -> bool {
            self.tickets.borrow().contains(ticket)
        }
    }

    impl Dispatcher for TestDispatcher {
        type UniqueId = u32;

        fn send_ticket(&self, ticket: Ticket) {
            self.tickets.borrow_mut().push(ticket);
        }

        fn id(&self) -> Self::UniqueId {
            self.id
        }
    }

    #[test]
    fn test_road_monitor() {
        let dispatcher1 = Rc::new(TestDispatcher { id: 1, tickets: Default::default()});
        let mut road_monitor: RoadMonitor<TestDispatcher> = RoadMonitor::from(123);
        road_monitor.add_ticket_dispatcher(dispatcher1.clone());

        let unix_car = Rc::new(Car::from("UN1X".to_owned()));

        road_monitor.record_observation(unix_car.clone(), 8, 0, 60);
        road_monitor.record_observation(unix_car.clone(), 9, 45, 60);

        let ticket = Ticket { car: unix_car.clone(), road_id: 123, mile1: 8, timestamp1: 0, mile2: 9, timestamp2: 45 };

        assert_eq!(ticket.calculate_speed(), 8000);

        assert!(dispatcher1.has_ticket(&ticket));
    }
}