pub fn range_inclusive_size<T>(range: &std::ops::RangeInclusive<T>) -> Option<usize>
where
    T: Copy + Ord + Into<usize>,
{
    let start = (*range.start()).into();
    let end = (*range.end()).into();

    end.checked_sub(start)
        .map(|d| d + 1)
}

pub fn range_size<T>(range: &std::ops::Range<T>) -> Option<usize>
where
    T: Copy + Ord + Into<usize>,
{
    let start = range.start.into();
    let end = range.end.into();

    end.checked_sub(start)
}

pub fn range_inclusive_size_clone<T>(range: &std::ops::RangeInclusive<T>) -> Option<usize>
where
    T: Clone + Ord + Into<usize>,
{
    let start = range.start().clone().into();
    let end = range.end().clone().into();

    end.checked_sub(start)
        .map(|d| d + 1)
}

pub fn range_size_clone<T>(range: &std::ops::Range<T>) -> Option<usize>
where
    T: Clone + Ord + Into<usize>,
{
    let start = range.start.clone().into();
    let end = range.end.clone().into();

    end.checked_sub(start)
}
