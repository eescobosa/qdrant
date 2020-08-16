use crate::collection::{Collection, OperationResult};
use crate::segment_manager::holders::segment_holder::SegmentHolder;
use segment::segment_constructor::simple_segment_constructor::build_simple_segment;
use std::path::Path;
use crate::operations::types::CollectionConfig;
use crate::wal::SerdeWal;
use crate::operations::CollectionUpdateOperations;
use wal::WalOptions;
use std::sync::{Arc, RwLock};
use tokio::runtime::Handle;
use crate::segment_manager::simple_segment_searcher::SimpleSegmentSearcher;
use crate::segment_manager::simple_segment_updater::SimpleSegmentUpdater;
use crossbeam_channel::unbounded;
use crate::update_handler::update_handler::{UpdateHandler, Optimizer};


pub fn build_simple_collection(
    num_segments: usize, // from config
    _segment_path: &Path, // from service
    wal_path: &Path,  // from config
    wal_options: &WalOptions,  // from config
    config: &CollectionConfig,  //  from user
    search_runtime: Handle,  // from service
    update_runtime: Handle,  // from service
    optimizers: Arc<Vec<Box<Optimizer>>>,
) -> OperationResult<Collection> {
    let mut segment_holder = SegmentHolder::new();

    for _sid in 0..num_segments {
        let segment = build_simple_segment(config.vector_size, config.distance.clone());
        segment_holder.add(segment);
    }

    let segment_holder = Arc::new(RwLock::new(segment_holder));

    let wal: SerdeWal<CollectionUpdateOperations> = SerdeWal::new(wal_path.to_str().unwrap(), wal_options)?;

    let searcher = SimpleSegmentSearcher::new(
        segment_holder.clone(),
        search_runtime,
        config.distance.clone(),
    );

    let updater = SimpleSegmentUpdater::new(segment_holder.clone());

    let (tx, rx) = unbounded();

    let update_handler = Arc::new(UpdateHandler::new(
        optimizers,
        rx,
        update_runtime.clone()
    ));

    let collection = Collection {
        wal: Arc::new(RwLock::new(wal)),
        searcher: Arc::new(searcher),
        update_handler,
        updater: Arc::new(updater),
        runtime_handle: update_runtime,
        update_sender: tx
    };

    Ok(collection)
}

