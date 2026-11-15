use super::*;

#[test]
fn exhausted_revision_does_not_consume_batch_or_mutate_samples() {
    let mut stream = PlotStream::new(1, 16).unwrap();
    stream.append_once(1, &[[1.0; 2]]).unwrap();
    stream.revision = u64::MAX;
    assert_eq!(
        stream.append_once(2, &[[2.0; 2]]),
        Err(StreamError::RevisionExhausted)
    );
    assert_eq!(stream.last_batch, Some(1));
    assert_eq!(stream.as_slices(), (&[[1.0; 2]][..], &[][..]));
    assert_eq!(stream.clear(), Err(StreamError::RevisionExhausted));
    assert_eq!(stream.len(), 1);
}
