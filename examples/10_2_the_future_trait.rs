/*
A future is anything that implements the Future trait
This trait allows for something to be used asynchronously
*/

use std::pin::Pin;
use std::task::{Context, Poll};

//the future trait
pub trait Future {
    type Output;
    /*
    This is the type that will be returned by the future when it completes
    i.e. the food we get after finishing cooking
    Or an nice job after learning programming :D

    More seriously, this could be the API response we get after making a network request
     */

    fn poll(self: Pin<&mut Self>, cx: &mut Context)
        -> Poll<Self::Output>;
    /*
    The poll function is how we check if a future is done
    The basic gist of it is this
    The Poll enum has two subtypes, "Ready" and "Pending"
    When the poll function is implemented, logic is written about what defines if a future is completed, or if it is still in progress (will see below)
    If it is done, Poll::Ready is returned
    If it is still in progress, Poll::Pending is returned

    The "Pin" in the parameter allows this to be used in conjunction with async, though that is not covered here. See async book and documentation for more on this.
     */
}

fn main(){}
//kept just to make the errors go away