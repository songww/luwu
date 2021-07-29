pub use luwu::processors::{Saga, SagaStep};

impl Saga {
    // Add add a saga step
    async fn add(&mut self, on_committing: string, on_reverting: String, payload: String) -> Result<(), ()> {
    	debug!("saga {} add on_committing:`{}` on_revert:`{}` {:?}", self.gid, on_committing, on_reverting, postData);
    	let step = SagaStep{
    		on_committing,
    		on_reverting,
    		payload,
    	};
    	self.steps.push(step);
        Ok(())
    }
    
    // Submit the saga transaction
    async fn submit(&self) -> Result<(), ()> {
    	debug!("committing {} body: {:?}", self.gid, &self.payload);
    	// resp, err := common.RestyClient.R().SetBody(&s.SagaData).Post(fmt.Sprintf("%s/submit", s.Server))
    	// return CheckDtmResponse(resp, err)
        Ok(())
    }
}
