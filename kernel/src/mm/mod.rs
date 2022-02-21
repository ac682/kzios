mod heap_allocator;
mod address;

pub fn init()
{
    heap_allocator::init_heap();
}