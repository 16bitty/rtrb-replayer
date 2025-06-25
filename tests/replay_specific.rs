use rtrb::RingBuffer;

#[test]
fn resend_window_blocking() {
    let (mut p, mut c) = RingBuffer::new(5, 2);
    
    // Should be able to write 3 messages (5-2)
    p.push(1).unwrap();
    p.push(2).unwrap();
    p.push(3).unwrap();
    
    // Next push should fail (only 2 protected slots left)
    assert!(p.push(4).is_err());
    
    // After reading one item, should have space for one more
    assert_eq!(c.pop().unwrap(), 1);
    assert!(p.push(4).is_ok());
}

#[test]
fn history_basic_access() {
    let (mut p, c) = RingBuffer::new(5, 0);
    
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    let history = c.history();
    
    // Check history contains all items
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(history.start_index()), Some(&10));
    assert_eq!(history.get(history.start_index() + 1), Some(&20));
    assert_eq!(history.get(history.start_index() + 2), Some(&30));
}

#[test]
fn history_after_pop() {
    let (mut p, mut c) = RingBuffer::new(5, 0);
    
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    // Pop first item
    assert_eq!(c.pop().unwrap(), 10);
    
    let history = c.history();
    
    // Check history excludes popped item
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(history.start_index()), Some(&20));
    assert_eq!(history.get(history.start_index() + 1), Some(&30));
}

#[test]
fn history_after_producer_advance() {
    let (mut p, c) = RingBuffer::new(3, 0);
    
    p.push(1).unwrap();
    let history1 = c.history();
    assert_eq!(history1.len(), 1);
    
    p.push(2).unwrap();
    let history2 = c.history();
    assert_eq!(history2.len(), 2);
    
    p.push(3).unwrap();
    let history3 = c.history();
    assert_eq!(history3.len(), 3);
}

#[test]
fn history_wraparound() {
    let (mut p, mut c) = RingBuffer::new(3, 0);
    
    // Fill buffer completely
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    // Create space at beginning
    assert_eq!(c.pop().unwrap(), 10);
    assert_eq!(c.pop().unwrap(), 20);
    
    // Cause wrap-around
    p.push(40).unwrap();
    p.push(50).unwrap();
    
    let history = c.history();
    
    // Check history contains all items
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(history.start_index()), Some(&30));
    assert_eq!(history.get(history.start_index() + 1), Some(&40));
    assert_eq!(history.get(history.start_index() + 2), Some(&50));
}
 #[test]
fn history_iteration() {
    let (mut p, c) = RingBuffer::new(5, 0);
    
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    let history = c.history();
    let mut iter = history.iter();
    
    assert_eq!(iter.next(), Some(&10));
    assert_eq!(iter.next(), Some(&20));
    assert_eq!(iter.next(), Some(&30));
    assert_eq!(iter.next(), None);
    
    // Check size_hint is accurate
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn history_with_resend_window() {
    let (mut p, mut c) = RingBuffer::new(5, 2);
    
    // Fill buffer to resend limit (5-2=3 messages)
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    // Next push should fail due to resend window
    assert!(p.push(40).is_err());
    
    let history = c.history();
    
    // History should contain all written messages
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(history.start_index()), Some(&10));
    assert_eq!(history.get(history.start_index() + 1), Some(&20));
    assert_eq!(history.get(history.start_index() + 2), Some(&30));
    
    assert_eq!(c.pop().unwrap(), 10);
    p.push(40).unwrap();
    
    let history = c.history();
    
    // History should reflect new state
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(history.start_index()), Some(&20));
    assert_eq!(history.get(history.start_index() + 1), Some(&30));
    assert_eq!(history.get(history.start_index() + 2), Some(&40));
}

#[test]
fn history_indexing() {
    let (mut p, mut c) = RingBuffer::new(3, 0);
    
    let initial_head = c.head();
    
    p.push(10).unwrap();
    p.push(20).unwrap();
    assert_eq!(c.pop().unwrap(), 10);
    p.push(30).unwrap();
    
    let history = c.history();
    
    assert_eq!(history.start_index(), initial_head + 1);
    assert_eq!(history.end_index(), initial_head + 3);
    
    // Check items at specific indices
    assert_eq!(history.get(initial_head), None);
    assert_eq!(history.get(initial_head + 1), Some(&20));
    assert_eq!(history.get(initial_head + 2), Some(&30));
}

#[test]
fn history_empty_buffer() {
    let (_, c) = RingBuffer::<i32>::new(3, 0);
    let history = c.history();
    
    // Should be empty but valid
    assert_eq!(history.len(), 0);
    assert!(history.is_empty());
    assert_eq!(history.start_index(), c.head());
    assert_eq!(history.end_index(), c.head());
    assert_eq!(history.iter().next(), None);
}

#[test]
fn history_full_buffer() {
    let (mut p, c) = RingBuffer::new(3, 0);
    
    p.push(10).unwrap();
    p.push(20).unwrap();
    p.push(30).unwrap();
    
    let history = c.history();
    
    // Should contain all items
    assert_eq!(history.len(), 3);
    let items: Vec<i32> = history.iter().copied().collect();
    assert_eq!(items, vec![10, 20, 30]);
}

#[test]
fn history_position_access() {
    let (mut p, mut c) = RingBuffer::new(3, 0);
    
    let start_pos = c.head();
    
    p.push(100).unwrap();
    p.push(200).unwrap();
    
    // Access messages by their specific indices
    let history = c.history();
    assert_eq!(history.get(start_pos), Some(&100));
    assert_eq!(history.get(start_pos + 1), Some(&200));
    
    c.pop().unwrap();
    let history = c.history();
    assert_eq!(history.get(start_pos), None);
    assert_eq!(history.get(start_pos + 1), Some(&200));
}