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