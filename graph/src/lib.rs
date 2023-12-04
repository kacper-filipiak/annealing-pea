pub mod graph {

    use ndarray::Array2;
    use ndarray::Axis;
    use rand::Rng;
    use std::fs;

    pub struct Graph<T> {
        matrix: Box<Array2<T>>,
    }
    impl<T> std::ops::Index<(usize, usize)> for Graph<T> {
        type Output = T;

        fn index(&self, index: (usize, usize)) -> &T {
            return &self.matrix[index];
        }
    }
    impl<T> std::fmt::Display for Graph<T>
    where
        T: std::fmt::Display + Default,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let text = format!("{}", self.matrix);
            let default = format!(" {} ", T::default());
            write!(f, "{}", text.replace(default.as_str(), "-"))
        }
    }

    //impl Graph<u32> {
    //    fn get_if_connected(&self, v1: usize, v2: usize) -> Option<u32> {
    //        let weight = self.matrix[(v1, v2)];
    //        if weight == 0 {
    //            return None;
    //        } else {
    //            return Some(weight);
    //        }
    //    }
    //}
    //

    impl Graph<u32> {
        pub fn set_zero_to_max(&mut self) {
            for i in 0..self.matrix.dim().0 {
                for j in 0..self.matrix.dim().1 {
                    if self.matrix[(i, j)] == 0 {
                        self.matrix[(i, j)] = u32::MAX;
                    }
                }
            }
        }
    }

    impl<T> Graph<T>
    where
        T: Clone
            + rand::distributions::uniform::SampleUniform
            + std::str::FromStr
            + std::cmp::Ord
            + Default
            + Copy
            + std::fmt::Display,
    {
        pub fn generate_random_complete_graph(
            n: usize,
            weight_range: std::ops::Range<T>,
            additional_edges: usize,
        ) -> Graph<T> {
            if n - 1 + additional_edges > (n * n - n) / 2 {
                panic!(
                    "Too many edges requested for {} vertex graph! {}/{}",
                    n,
                    n + additional_edges - 1,
                    (n * n - n) / 2
                )
            }
            let mut matrix = Box::new(Array2::<T>::default((n + 1, n + 1)));
            let mut rng = rand::thread_rng();
            for i in 2..=n {
                let index = rng.gen_range(1..i);
                let weigth = rng.gen_range(weight_range.start..weight_range.end);
                matrix[(i, index)] = weigth;
                matrix[(index, i)] = weigth;
            }
            let mut graph = Graph { matrix };
            for _ in 0..additional_edges {
                loop {
                    let v1: usize = rng.gen_range(2..=n);
                    let v2: usize = rng.gen_range(1..v1);
                    let weight: T = rng.gen_range(weight_range.start..weight_range.end);
                    let success: bool;
                    success = graph.add_edge_if_not_exists(v1, v2, weight);
                    if success {
                        break;
                    }
                }
            }
            return graph;
        }

        pub fn read_graph_from_file(filename: &str) -> Graph<T> {
            let contents: String =
                fs::read_to_string(filename).expect("Should have been able to read file");

            let lines: Vec<Vec<&str>> = contents.lines().map(|x| x.split(',').collect()).collect();
            let number_of_vertex: usize = lines.first().unwrap().first().unwrap().parse().unwrap();
            let mut matrix = Box::new(Array2::<T>::default((
                number_of_vertex + 1,
                number_of_vertex + 1,
            )));
            let edges: &[Vec<&str>] = &lines[1..];
            for e in edges {
                let length = e.len();
                if length == 3 {
                    let v1: usize = e[0].trim().parse().expect("Expected vertex number");
                    let v2: usize = e[1].trim().parse().expect("Expected vertex number");
                    let w: T = match e[2].trim().parse() {
                        Ok(x) => x,
                        Err(_) => panic!("Expected weight"),
                    };
                    matrix[[v1, v2]] = w;
                    matrix[[v2, v1]] = w;
                } else {
                    print!("e.len(): {length}\n");
                }
            }
            return Graph { matrix };
        }

        pub fn read_graph_from_file_full_table(filename: &str) -> Graph<T> {
            let contents: String =
                fs::read_to_string(filename).expect("Should have been able to read file");

            let lines: Vec<Vec<&str>> = contents
                .lines()
                .map(|x| {
                    x.trim()
                        .split(' ')
                        .filter(|x| !x.trim().is_empty())
                        .collect()
                })
                .collect();
            let number_of_vertex: usize = lines.first().unwrap().first().unwrap().parse().unwrap();
            let mut matrix = Box::new(Array2::<T>::default((
                number_of_vertex + 1,
                number_of_vertex + 1,
            )));
            for i in 0..number_of_vertex {
                for j in 0..number_of_vertex {
                    matrix[(i + 1, j + 1)] = match lines[i + 1][j].trim().parse() {
                        Ok(x) => x,
                        Err(_) => {
                            panic!(
                                "Expected weight byt got \"{}\", on index ({}, {})",
                                lines[i][j], i, j
                            )
                        }
                    };
                }
            }
            return Graph { matrix };
        }

        pub fn save_to_file(&self, filename: &str) {
            let number_of_vertex = self.number_of_vertex();
            let mut content: Box<String> = Box::new(String::new());
            content.push_str(&format!("{number_of_vertex}\n"));
            for i in 1..=number_of_vertex {
                for j in 1..i {
                    if self.connected((i, j)) {
                        content.push_str(&format!("{}, {}, {}\n", i, j, self.matrix[(i, j)]));
                    }
                }
            }
            print!("{content}");
            fs::write(filename, content.as_bytes())
                .expect("Should be able to write file {filename}\n");
        }

        pub fn connected(&self, id: (usize, usize)) -> bool {
            self.matrix[id] != T::default()
        }

        pub fn number_of_vertex(&self) -> usize {
            return self.matrix.len_of(Axis(0)) - 1;
        }

        fn add_edge_if_not_exists(&mut self, v1: usize, v2: usize, weight: T) -> bool {
            if self.matrix[(v1, v2)] == T::default() {
                self.matrix[(v1, v2)] = weight;
                self.matrix[(v2, v1)] = weight;
                return true;
            } else {
                return false;
            }
        }

        pub fn distance(&self, v1: usize, v2: usize) -> T {
            self.matrix[(v1, v2)]
        }
    }
    impl Graph<u32> {
        pub fn distance_vec(&self, v: &Vec<usize>) -> u32 {
            if v.len() < 2 {
                return 0;
            }
            let mut v1 = v.first().unwrap().clone();
            let mut sum = 0;
            for v2 in 1..v.len() {
                sum = sum + self.distance(v1, v[v2]);
                v1 = v[v2];
            }
            sum
        }
        pub fn distance_cycle(&self, v: &Vec<usize>) -> u32 {
            self.distance_vec(v)
                + self.distance(v.first().unwrap().clone(), v.last().unwrap().clone())
        }
    }
}
