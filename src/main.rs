// Система парковки. Консольная программа на Rust, весь код в одном файле.

use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

// ---------- Структуры ----------

struct User { id: u32, name: String, balance: f64 }
struct Car { plate: String, owner_id: u32 }
// Парковочное место. Если plate пустой, то место свободно
struct Spot { plate: String, start: u64 }
struct Parking { name: String, price: f64, spots: Vec<Spot> }
// Система управления: хранит всё вместе
struct ParkingSystem {
    parking: Option<Parking>, // None, пока парковка не создана
    users: Vec<User>,
    cars: Vec<Car>,
}

// ---------- Ввод с консоли ----------

fn input(text: &str) -> String {
    print!("{}", text);
    io::stdout().flush().unwrap();
    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();
    s.trim().to_string()
}

// Читаем число. Если ввели не число, получится 0 (дальше это проверяется)
fn number(text: &str) -> f64 {
    input(text).replace(",", ".").parse().unwrap_or(0.0)
}

// Номер машины: без пробелов и большими буквами
fn fix_plate(s: &str) -> String {
    s.replace(" ", "").to_uppercase()
}

// ---------- Методы системы. Каждый возвращает текст для вывода ----------

impl ParkingSystem {
    fn now(&self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    // 1. Создание парковки
    fn create_parking(&mut self, name: String, count: u32, price: f64) -> String {
        if self.parking.is_some() {
            return "Парковка уже создана".to_string();
        }
        if name.is_empty() || count == 0 || price <= 0.0 {
            return "Ошибка: нужно название, места и цена больше 0".to_string();
        }
        let mut spots = Vec::new();
        for _ in 0..count {
            spots.push(Spot { plate: String::new(), start: 0 });
        }
        self.parking = Some(Parking { name, price, spots });
        "Парковка создана".to_string()
    }

    fn add_user(&mut self, name: String, balance: f64) -> String {
        if name.is_empty() || balance < 0.0 {
            return "Ошибка: пустое имя или отрицательный баланс".to_string();
        }
        let id = self.users.len() as u32 + 1;
        self.users.push(User { id, name, balance });
        format!("Пользователь создан, его id = {}", id)
    }

    fn add_car(&mut self, user_id: u32, plate: &str) -> String {
        let plate = fix_plate(plate);
        if plate.is_empty() {
            return "Ошибка: пустой номер".to_string();
        }
        if user_id == 0 || user_id as usize > self.users.len() {
            return "Пользователь не найден".to_string();
        }
        for car in &self.cars {
            if car.plate == plate {
                return "Такая машина уже есть".to_string();
            }
        }
        self.cars.push(Car { plate, owner_id: user_id });
        "Машина добавлена".to_string()
    }

    fn top_up(&mut self, user_id: u32, amount: f64) -> String {
        if user_id == 0 || user_id as usize > self.users.len() {
            return "Пользователь не найден".to_string();
        }
        if amount <= 0.0 {
            return "Сумма должна быть больше 0".to_string();
        }
        let user = &mut self.users[user_id as usize - 1];
        user.balance += amount;
        format!("Новый баланс: {:.2}", user.balance)
    }

    // 2. Парковка машины
    fn park_car(&mut self, plate: &str) -> String {
        let plate = fix_plate(plate);
        let now = self.now();

        let mut registered = false;
        for car in &self.cars {
            if car.plate == plate {
                registered = true;
            }
        }
        if !registered {
            return "Машина не зарегистрирована".to_string();
        }
        let parking = match self.parking.as_mut() {
            Some(p) => p,
            None => return "Парковка ещё не создана".to_string(),
        };
        for spot in &parking.spots {
            if spot.plate == plate {
                return "Машина уже на парковке".to_string();
            }
        }
        // ищем первое свободное место
        for (i, spot) in parking.spots.iter_mut().enumerate() {
            if spot.plate.is_empty() {
                spot.plate = plate;
                spot.start = now;
                return format!("Машина встала на место №{}", i + 1);
            }
        }
        "Свободных мест нет".to_string()
    }

    // 3. Выезд и оплата с учётом времени
    fn leave_and_pay(&mut self, plate: &str) -> String {
        let plate = fix_plate(plate);
        let now = self.now();
        let parking = match self.parking.as_mut() {
            Some(p) => p,
            None => return "Парковка ещё не создана".to_string(),
        };
        for spot in parking.spots.iter_mut() {
            if spot.plate == plate {
                // минуты (неполная минута считается за целую, минимум 1)
                let minutes = ((now - spot.start + 59) / 60).max(1);
                let cost = minutes as f64 * parking.price / 60.0;

                // находим владельца машины
                let mut owner = 0;
                for car in &self.cars {
                    if car.plate == plate {
                        owner = car.owner_id;
                    }
                }
                let user = &mut self.users[owner as usize - 1];
                if user.balance < cost {
                    return format!("Не хватает денег: нужно {:.2}, на балансе {:.2}", cost, user.balance);
                }
                user.balance -= cost;
                spot.plate = String::new(); // место освободилось
                return format!(
                    "Чек: время {} мин, к оплате {:.2}, остаток {:.2}",
                    minutes, cost, user.balance
                );
            }
        }
        "Такой машины нет на парковке".to_string()
    }

    fn show_parking(&self) {
        match &self.parking {
            None => println!("Парковка ещё не создана"),
            Some(p) => {
                println!("Парковка \"{}\", цена {:.2} за час", p.name, p.price);
                for (i, spot) in p.spots.iter().enumerate() {
                    if spot.plate.is_empty() {
                        println!("  Место {}: свободно", i + 1);
                    } else {
                        println!("  Место {}: {} ({} мин)", i + 1, spot.plate, (self.now() - spot.start) / 60);
                    }
                }
            }
        }
    }

    fn show_users(&self) {
        for user in &self.users {
            print!("  #{} {}, баланс {:.2}, машины:", user.id, user.name, user.balance);
            for car in &self.cars {
                if car.owner_id == user.id {
                    print!(" {}", car.plate);
                }
            }
            println!();
        }
    }
}

// ---------- Главная функция ----------

fn main() {
    let mut s = ParkingSystem { parking: None, users: Vec::new(), cars: Vec::new() };

    loop {
        println!("\n===== ПАРКОВКА =====");
        println!("1. Создать парковку\n2. Добавить пользователя\n3. Добавить машину пользователю");
        println!("4. Пополнить баланс\n5. Припарковать машину\n6. Выехать и оплатить");
        println!("7. Показать парковку\n8. Показать пользователей\n0. Выход");

        match input("Ваш выбор: ").as_str() {
            "1" => {
                let name = input("Название парковки: ");
                let count = number("Количество мест: ") as u32;
                let price = number("Цена за час: ");
                println!("{}", s.create_parking(name, count, price));
            }
            "2" => {
                let name = input("Имя: ");
                let balance = number("Начальный баланс: ");
                println!("{}", s.add_user(name, balance));
            }
            "3" => {
                let id = number("id пользователя: ") as u32;
                let plate = input("Номер машины: ");
                println!("{}", s.add_car(id, &plate));
            }
            "4" => {
                let id = number("id пользователя: ") as u32;
                let amount = number("Сумма: ");
                println!("{}", s.top_up(id, amount));
            }
            "5" => println!("{}", s.park_car(&input("Номер машины: "))),
            "6" => println!("{}", s.leave_and_pay(&input("Номер машины: "))),
            "7" => s.show_parking(),
            "8" => s.show_users(),
            "0" => break,
            _ => println!("Нет такого пункта меню"),
        }
    }
}
