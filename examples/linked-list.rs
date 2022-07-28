use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct Node<V>
where
    V: Display,
{
    inner: V,
    next: Option<Rc<RefCell<Self>>>,
}

impl<V> Display for Node<V>
where
    V: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut a = format!("{},", self.inner);

        if let Some(nxt) = &self.next {
            let b = format!("{}", nxt.borrow());
            a = a + &b;
        } else {
            a = a + "null";
        }
        write!(f, "{}", a)
    }
}

fn main() {
    let mut i = 0;
    let head = Node::<i32> {
        inner: i.to_owned(),
        next: None,
    };

    let mut times = 10;
    let head_p = Rc::new(RefCell::new(head));
    let mut p = head_p.clone();
    while times > 0 {
        i += 1;
        let new_node = Node::<i32> {
            inner: i,
            next: None,
        };
        p.borrow_mut().next = Some(Rc::new(RefCell::new(new_node)));
        let cp = p.clone();
        match &cp.borrow().next {
            Some(nxt) => {
                p = Rc::clone(&nxt);
            }
            _ => break,
        }
        times -= 1;
    }
    // output
    println!("linked list: {}", head_p.borrow_mut());
}
