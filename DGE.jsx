import React, { useState, useMemo } from 'react';
import { Sparkles, Zap, Lock, DollarSign, ChevronRight, CheckCircle, Clock, ZapOff, Scale, XCircle, Slash } from 'lucide-react';

// --- DGE CORE LOGIC (Simulating gitbook.rs or smart contract constants) ---

// FUNDING LIMITS & SCALING
const MIN_GRANT_CAP = 1000; // Base grant awarded at MIN_D_METRIC_REQUIRED
const MAX_GRANT_CAP = 10000; // Max grant awarded at D_METRIC_SCALE
const D_METRIC_SCALE = 250; // Max D-Metric required for $10,000 capacity
const MIN_D_METRIC_REQUIRED = 10; // Minimum D-Metric to even submit a proposal
const D_METRIC_BOOST = 20; // Points added to D-Metric upon successful grant completion
const D_METRIC_PENALTY = 10; // Points deducted upon failure/slashing

// SECURITY & BONDING
const BUILDER_BOND_USD = 300; // Required stake amount to submit a proposal (USD)
const GOVERNANCE_TOKEN_PRICE = 0.5; // Simulated $DEPTH Token Price
const MIN_GRANT_REQUESTED = 100; // Minimum amount a founder can request

// ADAPTIVE QUORUM (AQ) LOGIC
const AQ_BASE_PERCENTAGE = 0.05; // 5.00% - Absolute minimum voter turnout required.
const AQ_SENSITIVITY_MULTIPLIER = 0.00000001; // 1e-8 - Rises 10% for every 10M $DEPTH in circulation.
const AQ_QUORUM_CEILING = 0.15; // 15.00% - Hard cap on the quorum requirement.

/**
 * Calculates the maximum funding cap based on the D-Metric (Credibility-based Allocation).
 * @param {number} dMetric - The founder's Depth Metric (0-250).
 * @returns {number} The maximum USD grant allocation right.
 */
const calculateFundingCap = (dMetric) => {
  if (dMetric < MIN_D_METRIC_REQUIRED) {
    return 0; 
  }
  
  if (dMetric >= D_METRIC_SCALE) return MAX_GRANT_CAP;

  // Linear interpolation scales funding capacity from $1,000 (at 10 D-Pnt) up to $10,000 (at 250 D-Pnt).
  const scaleRange = D_METRIC_SCALE - MIN_D_METRIC_REQUIRED;
  const capRange = MAX_GRANT_CAP - MIN_GRANT_CAP;
  
  const dMetricAdjusted = dMetric - MIN_D_METRIC_REQUIRED;
  
  return Math.floor(MIN_GRANT_CAP + capRange * (dMetricAdjusted / scaleRange));
};

/**
 * Calculates the required Builder Bond stake in $DEPTH tokens.
 */
const getBuilderBondAmount = (tokenPrice) => {
  return Math.ceil(BUILDER_BOND_USD / tokenPrice);
};

// --- APPLICATION STATE AND COMPONENTS ---

const App = () => {
  // STARTING D-Metric at the minimum eligibility floor (10)
  const [dMetric, setDMetric] = useState(MIN_D_METRIC_REQUIRED);
  const [depthSupply, setDepthSupply] = useState(10000000); 
  const [grantRequested, setGrantRequested] = useState(MIN_GRANT_CAP); 
  
  // Governance States
  const [isSubmitted, setIsSubmitted] = useState(false); // Proposal submitted, awaiting initial DAO vote
  const [isApproved, setIsApproved] = useState(false);   // Initial DAO vote passed
  const [isSlashingVoteActive, setIsSlashingVoteActive] = useState(false); // DAO voting on penalty
  const [milestoneProgress, setMilestoneProgress] = useState(0); // Count of milestones funded (0 to 4)
  const [message, setMessage] = useState(null);

  // Constants derived from DGE Logic
  const maxCap = useMemo(() => calculateFundingCap(dMetric), [dMetric]);
  const requiredBond = useMemo(() => getBuilderBondAmount(GOVERNANCE_TOKEN_PRICE), []);
  
  // Adaptive Quorum Calculation
  const adaptiveQuorum = useMemo(() => {
    const calculatedQuorum = AQ_BASE_PERCENTAGE + (depthSupply * AQ_SENSITIVITY_MULTIPLIER);
    return Math.min(calculatedQuorum, AQ_QUORUM_CEILING); 
  }, [depthSupply]);

  // Grant Structure 
  const GRANT_MILESTONES = useMemo(() => [
    { name: "Phase 1: Proof of Concept", percent: 25 },
    { name: "Phase 2: Alpha Launch & Testnet", percent: 30 },
    { name: "Phase 3: Community Feedback Loop", percent: 25 },
    { name: "Phase 4: Mainnet Launch & Review", percent: 20 },
  ], []);
  
  // Check if requested grant is within the founder's capacity
  const grantCapStatus = grantRequested > maxCap ? 'Exceeds Cap' : 'Within Cap';

  // Eligibility Check
  const isEligibleForSubmission = dMetric >= MIN_D_METRIC_REQUIRED && grantCapStatus === 'Within Cap' && grantRequested >= MIN_GRANT_REQUESTED;

  // Total funds released calculation
  const fundsReleased = useMemo(() => {
    let released = 0;
    if (!isApproved && !isSlashingVoteActive) return 0; // Only funds released after initial approval
    
    // Calculate funds released based on completed milestones
    for (let i = 0; i < milestoneProgress; i++) {
      released += grantRequested * (GRANT_MILESTONES[i].percent / 100);
    }
    return released;
  }, [grantRequested, isApproved, isSlashingVoteActive, milestoneProgress, GRANT_MILESTONES]);

  const fundsRemaining = grantRequested - fundsReleased;

  // Resets all progress if inputs change
  const handleInputChange = (setter, value) => {
    setter(value);
    setIsSubmitted(false);
    setIsApproved(false);
    setIsSlashingVoteActive(false);
    setMilestoneProgress(0);
    setMessage(null);
  };

  const showMessage = (content, type) => {
    const typeClasses = {
      pending: "bg-indigo-100 text-indigo-700",
      success: "bg-emerald-100 text-emerald-700",
      milestone: "bg-yellow-100 text-yellow-700",
      error: "bg-red-100 text-red-700"
    };

    setMessage({ content, type, classes: typeClasses[type] });
    setTimeout(() => setMessage(null), 5000);
  };

  // Step 1: Founder submits proposal and pays the bond
  const handleProposalSubmission = () => {
    if (isEligibleForSubmission) {
      setIsSubmitted(true);
      showMessage(
        <>
          <Clock className="w-5 h-5 mr-2" />
          Proposal Submitted! **{requiredBond.toLocaleString()} $DEPTH Bond** staked. Awaiting 7-day DAO vote.
        </>, 'pending'
      );
    } else {
      showMessage(
        <>
          <ZapOff className="w-5 h-5 mr-2" />
          Submission Rejected: Check D-Metric Floor (**{MIN_D_METRIC_REQUIRED}**), ensure request is above **${MIN_GRANT_REQUESTED}**, or is within DGE Cap.
        </>, 'error'
      );
    }
  };

  // Step 2: Simulate the DAO vote passing (Quorum and Majority Met) or failing
  const simulateDaoVote = (passed) => {
    if (isSubmitted && !isApproved) {
        if (passed) {
            setIsApproved(true);
            setMilestoneProgress(1); // Auto-release Phase 1 Tranche on approval
            showMessage(
                <>
                    <CheckCircle className="w-5 h-5 mr-2" />
                    DAO Vote Passed! **Adaptive Quorum ({(adaptiveQuorum * 100).toFixed(2)}%)** met. First tranche released.
                </>, 'success'
            );
        } else {
            // Failed vote: founder loses bond, process resets
            setIsSubmitted(false);
            showMessage(
                <>
                    <XCircle className="w-5 h-5 mr-2" />
                    DAO Vote Failed! **{requiredBond.toLocaleString()} $DEPTH Bond** is forfeited. Proposal cancelled.
                </>, 'error'
            );
        }
    }
  };

  // Milestone Success Path: Releases next tranche and checks for D-Metric boost
  const handleMilestoneSuccess = () => {
    if (isApproved && milestoneProgress < GRANT_MILESTONES.length) {
      const isFinalMilestone = (milestoneProgress + 1) === GRANT_MILESTONES.length;
      
      setMilestoneProgress(prev => prev + 1);

      if (isFinalMilestone) {
        // SUCCESS: Apply D-Metric boost upon final completion
        const newDMetric = Math.min(D_METRIC_SCALE, dMetric + D_METRIC_BOOST);
        setDMetric(newDMetric);
        
        showMessage(
          <>
            <Zap className="w-5 h-5 mr-2" />
            Project Complete! All funds released. Founder D-Metric increased by **{D_METRIC_BOOST} Points** (New D-Metric: **{newDMetric}**), increasing future grant capacity!
          </>, 'success'
        );
      } else {
        // Standard milestone release message
        showMessage(
          <>
            <ChevronRight className="w-5 h-5 mr-2" />
            Milestone {milestoneProgress} Proof-of-Work Approved. Tranche released!
          </>, 'milestone'
        );
      }
    }
  };

  // Milestone Failure Path: Founder defaults, initiates slashing vote
  const handleFounderDefault = () => {
    if (isApproved && !isSlashingVoteActive && milestoneProgress < GRANT_MILESTONES.length) {
        setIsSlashingVoteActive(true);
        showMessage(
            <>
                <XCircle className="w-5 h-5 mr-2" />
                FOUNDER DEFAULTED: PoW not delivered for Milestone {milestoneProgress + 1}. DAO slashing vote now active!
            </>, 'error'
        );
    }
  };

  // DAO Slashing Vote Resolution
  const simulateSlashingVote = (slash) => {
    setIsSlashingVoteActive(false);
    setIsApproved(false); // Grant process terminates regardless of outcome

    if (slash) {
        // SLASHED: Lose bond, lose D-Metric
        const newDMetric = Math.max(MIN_D_METRIC_REQUIRED, dMetric - D_METRIC_PENALTY);
        setDMetric(newDMetric);
        showMessage(
            <>
                <ZapOff className="w-5 h-5 mr-2" />
                SLASHED: DAO penalized founder. **{requiredBond.toLocaleString()} $DEPTH Bond** forfeited. D-Metric reduced by **{D_METRIC_PENALTY} Points** (New D-Metric: **{newDMetric}**). Grant terminated.
            </>, 'error'
        );
    } else {
        // FORGIVEN: Lose grant, keep bond/D-Metric
        showMessage(
            <>
                <Scale className="w-5 h-5 mr-2" />
                FORGIVEN: DAO voted against slashing. Grant cancelled, **Bond is returned**, D-Metric is unchanged. Grant terminated.
            </>, 'success'
        );
    }
    setMilestoneProgress(milestoneProgress); // Maintain current released funds on termination
  };


  return (
    <div className="min-h-screen bg-gray-50 p-4 sm:p-8 font-['Inter']">
      <style>{`
        /* Standard Slider Styling */
        .range-slider {
            -webkit-appearance: none;
            width: 100%;
            height: 8px;
            background: #e0e0e0;
            outline: none;
            opacity: 0.7;
            -webkit-transition: .2s;
            transition: opacity .2s;
            border-radius: 4px;
        }
        .range-slider::-webkit-slider-thumb {
            -webkit-appearance: none;
            appearance: none;
            width: 20px;
            height: 20px;
            border-radius: 50%;
            background: #2563EB;
            cursor: pointer;
            box-shadow: 0 0 5px rgba(0,0,0,0.2);
        }
        .range-slider::-moz-range-thumb {
            width: 20px;
            height: 20px;
            border-radius: 50%;
            background: #2563EB;
            cursor: pointer;
            box-shadow: 0 0 5px rgba(0,0,0,0.2);
        }
        /* Error Styling */
        .max-cap-exceed {
            border: 2px solid #ef4444 !important;
            color: #ef4444 !important;
        }
      `}</style>

      <div className="max-w-4xl mx-auto">
        <header className="text-center mb-10 p-6 bg-white rounded-xl shadow-lg border-b-4 border-indigo-500">
          <h1 className="text-3xl sm:text-4xl font-extrabold text-gray-900 flex items-center justify-center">
            <Zap className="w-8 h-8 mr-2 text-indigo-500" /> Depth Grant Engine Simulator (DGE)
          </h1>
          <p className="text-gray-500 mt-2">Modeling Credibility, Adaptive Quorum, and Failure Slashing</p>
        </header>
        
        {/* Message Box */}
        {message && (
            <div className={`p-4 rounded-xl shadow-md mb-6 flex items-center font-semibold ${message.classes}`}>
                {message.content}
            </div>
        )}

        {/* INPUTS PANEL */}
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-2xl font-semibold text-gray-800 mb-4 border-b pb-2">Profile & Protocol State Inputs</h2>
          
          <div className="grid md:grid-cols-2 gap-6">
            
            {/* D-Metric Input */}
            <div>
              <label className="text-lg font-medium text-gray-700 block mb-2 flex items-center">
                <Sparkles className="w-5 h-5 mr-2 text-yellow-500" /> Founder D-Metric: <span className="font-bold text-indigo-600 ml-2">{dMetric} / {D_METRIC_SCALE}</span>
              </label>
              <input
                type="range"
                min="0"
                max={D_METRIC_SCALE}
                value={dMetric}
                onChange={(e) => handleInputChange(setDMetric, parseInt(e.target.value))}
                className="range-slider"
              />
              <p className={`text-sm mt-1 font-medium ${dMetric < MIN_D_METRIC_REQUIRED ? 'text-red-600' : 'text-gray-500'}`}>
                **Commitment Floor**: Minimum **{MIN_D_METRIC_REQUIRED} D-Points** required for any allocation.
              </p>
            </div>

            {/* Depth Supply Input */}
            <div>
              <label className="text-lg font-medium text-gray-700 block mb-2 flex items-center">
                <Scale className="w-5 h-5 mr-2 text-purple-500" /> Total $DEPTH Supply (M): <span className="font-bold text-purple-600 ml-2">{(depthSupply / 1000000).toFixed(1)}M</span>
              </label>
              <input
                type="range"
                min="1000000"
                max="20000000"
                step="1000000"
                value={depthSupply}
                onChange={(e) => handleInputChange(setDepthSupply, parseInt(e.target.value))}
                className="range-slider"
              />
              <p className="text-sm text-gray-500 mt-1">Protocol's total supply proxy. Controls the **Adaptive Quorum**.</p>
            </div>
          </div>

          <div className="mt-6">
            <label className="text-lg font-medium text-gray-700 block mb-2 flex items-center">
              <DollarSign className="w-5 h-5 mr-2 text-green-500" /> Grant Requested (USD): <span className="font-bold text-green-600 ml-2">${grantRequested.toLocaleString()}</span>
            </label>
            <input
              type="number"
              min={MIN_GRANT_REQUESTED}
              max={MAX_GRANT_CAP}
              step="100"
              value={grantRequested}
              onChange={(e) => {
                const val = Math.min(MAX_GRANT_CAP, Math.max(MIN_GRANT_REQUESTED, parseInt(e.target.value) || 0));
                handleInputChange(setGrantRequested, val);
              }}
              className={`w-full p-3 border rounded-lg focus:ring-indigo-500 focus:border-indigo-500 transition duration-150 ${grantCapStatus === 'Exceeds Cap' ? 'max-cap-exceed' : 'border-gray-300'}`}
            />
            <p className={`text-sm mt-1 font-medium ${grantCapStatus === 'Exceeds Cap' ? 'text-red-600' : 'text-gray-500'}`}>
              Status: **{grantCapStatus}**. Max Cap based on D-Metric: **${maxCap.toLocaleString()}**.
            </p>
          </div>
        </div>

        {/* DGE OUTPUT & SECURITY CHECKS */}
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-2xl font-semibold text-gray-800 mb-4 border-b pb-2">DGE Security & Allocation Checks</h2>
          
          <div className="grid md:grid-cols-3 gap-6 text-center">
            
            <div className={`p-4 rounded-xl border-2 ${maxCap >= grantRequested && maxCap > 0 ? 'border-green-400 bg-green-50' : 'border-red-400 bg-red-50'}`}>
              <h3 className="font-bold text-lg mb-1 text-gray-700">1. DGE Allocation Right</h3>
              <p className="text-2xl font-extrabold text-green-600">${maxCap.toLocaleString()}</p>
              <p className="text-sm text-gray-500">Credibility ceiling based on D-Metric.</p>
            </div>

            <div className="p-4 rounded-xl border-2 border-indigo-400 bg-indigo-50">
              <h3 className="font-bold text-lg mb-1 text-gray-700">2. Builder Bond Cost</h3>
              <p className="text-2xl font-extrabold text-indigo-600">{requiredBond.toLocaleString()} $DEPTH</p>
              <p className="text-sm text-gray-500">Required stake (**$300 USD**), lost on slashing.</p>
            </div>

            <div className="p-4 rounded-xl border-2 border-purple-400 bg-purple-50">
              <h3 className="font-bold text-lg mb-1 text-gray-700">3. Adaptive Quorum (AQ)</h3>
              <p className="text-2xl font-extrabold text-purple-600">{(adaptiveQuorum * 100).toFixed(2)}%</p>
              <p className="text-sm text-gray-500">Minimum voter turnout for approval.</p>
            </div>
          </div>
        </div>

        {/* GOVERNANCE AND LIFECYCLE SIMULATOR */}
        <div className="bg-white p-6 rounded-xl shadow-lg mb-8">
          <h2 className="text-2xl font-semibold text-gray-800 mb-4 border-b pb-2">Grant Lifecycle: Proposal to Payout</h2>

          <div className="flex justify-between items-center mb-4">
            <div className="text-2xl font-extrabold text-gray-800">
              Released: <span className="text-emerald-600">${fundsReleased.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
            </div>
            <div className="text-xl font-medium text-gray-600">
              Remaining: <span className="text-red-500">${fundsRemaining.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
            </div>
          </div>
          
          <div className="w-full bg-gray-200 rounded-full h-3 mb-6">
            <div 
              className="bg-indigo-500 h-3 rounded-full transition-all duration-500" 
              style={{ width: `${(fundsReleased / grantRequested) * 100}%` }}
            ></div>
          </div>

          <div className="flex flex-col space-y-4">
            
            {/* Step 1: Submission */}
            {!isSubmitted && (
              <button
                onClick={handleProposalSubmission}
                disabled={!isEligibleForSubmission}
                className={`w-full p-3 rounded-xl font-bold transition duration-300 flex items-center justify-center ${
                  !isEligibleForSubmission
                    ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                    : 'bg-indigo-600 hover:bg-indigo-700 text-white shadow-md'
                }`}
              >
                <ChevronRight className="w-5 h-5 mr-2" />
                {!isEligibleForSubmission ? 
                  `INELIGIBLE: Need ${MIN_D_METRIC_REQUIRED} D-Points & Cap Met` 
                : `STEP 1: Submit Proposal & Pay ${requiredBond.toLocaleString()} $DEPTH Bond`}
              </button>
            )}

            {/* Step 2: Initial DAO Vote Simulation */}
            {isSubmitted && !isApproved && (
                <div className="p-4 border-2 border-dashed border-yellow-400 rounded-xl bg-yellow-50 text-center">
                    <p className="text-lg font-bold text-yellow-800 mb-3 flex items-center justify-center"><Clock className="w-5 h-5 mr-2"/> Proposal is currently in the 7-Day DAO Voting Period</p>
                    <div className="flex space-x-4">
                        <button
                            onClick={() => simulateDaoVote(true)}
                            className="p-3 w-1/2 rounded-xl font-bold transition duration-300 bg-emerald-600 hover:bg-emerald-700 text-white shadow-md flex items-center justify-center"
                        >
                            <CheckCircle className="w-5 h-5 mr-2" />
                            Simulate VOTE PASSED
                        </button>
                        <button
                            onClick={() => simulateDaoVote(false)}
                            className="p-3 w-1/2 rounded-xl font-bold transition duration-300 bg-red-600 hover:bg-red-700 text-white shadow-md flex items-center justify-center"
                        >
                            <XCircle className="w-5 h-5 mr-2" />
                            Simulate VOTE FAILED (Bond Forfeit)
                        </button>
                    </div>
                </div>
            )}
            
            {/* Step 3: Slashing Vote (Failure Resolution) */}
            {isSlashingVoteActive && (
                <div className="p-4 border-2 border-dashed border-red-400 rounded-xl bg-red-50 text-center">
                    <p className="text-lg font-bold text-red-800 mb-3 flex items-center justify-center"><Slash className="w-5 h-5 mr-2"/> Founder Defaulted! DAO Slashing Vote Active.</p>
                    <div className="flex space-x-4">
                        <button
                            onClick={() => simulateSlashingVote(true)}
                            className="p-3 w-1/2 rounded-xl font-bold transition duration-300 bg-gray-900 hover:bg-gray-700 text-white shadow-md flex items-center justify-center"
                        >
                            <ZapOff className="w-5 h-5 mr-2" />
                            VOTE TO **SLASH** (-10 D-Metric & Bond Loss)
                        </button>
                        <button
                            onClick={() => simulateSlashingVote(false)}
                            className="p-3 w-1/2 rounded-xl font-bold transition duration-300 bg-indigo-600 hover:bg-indigo-700 text-white shadow-md flex items-center justify-center"
                        >
                            <Scale className="w-5 h-5 mr-2" />
                            VOTE TO **FORGIVE** (Grant Terminated, No Penalty)
                        </button>
                    </div>
                </div>
            )}


            {/* Funding Tranches (Milestones) */}
            {isApproved && !isSlashingVoteActive && (
              <div className="border border-gray-200 rounded-xl p-4">
                <h3 className="text-xl font-semibold text-gray-700 mb-3">Milestone Progress (DGE Grant)</h3>
                <ol className="relative border-l border-gray-200 ml-4">
                  {GRANT_MILESTONES.map((milestone, index) => (
                    <li key={index} className="mb-8 ml-6">
                      <span className={`absolute flex items-center justify-center w-8 h-8 rounded-full -left-4 ring-4 ring-white ${milestoneProgress > index ? 'bg-emerald-500' : 'bg-gray-200'}`}>
                        {milestoneProgress > index ? <CheckCircle className="w-4 h-4 text-white" /> : <Lock className="w-4 h-4 text-gray-600" />}
                      </span>
                      <h4 className="flex items-center text-lg font-semibold text-gray-900">
                        {milestone.name}
                        {milestoneProgress > index && <span className="bg-emerald-100 text-emerald-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded ml-3">FUNDED</span>}
                        {milestoneProgress === index + 1 && index + 1 < GRANT_MILESTONES.length && <span className="bg-indigo-100 text-indigo-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded ml-3">CURRENT PHASE</span>}
                        {milestoneProgress === GRANT_MILESTONES.length && index === GRANT_MILESTONES.length - 1 && <span className="bg-indigo-500 text-white text-xs font-medium mr-2 px-2.5 py-0.5 rounded ml-3">PROJECT COMPLETE</span>}
                      </h4>
                      <p className="text-sm text-gray-500">Tranche Value: **${(milestone.percent / 100 * grantRequested).toLocaleString(undefined, { minimumFractionDigits: 2 })}** ({milestone.percent}%)</p>
                      
                      {/* Controls for the currently active milestone */}
                      {index + 1 === milestoneProgress && index + 1 <= GRANT_MILESTONES.length && (
                        <div className="mt-2 flex space-x-3">
                            <button
                                onClick={handleMilestoneSuccess}
                                className={`p-2 text-sm font-medium transition duration-150 rounded-lg flex items-center text-white 
                                    ${index + 1 < GRANT_MILESTONES.length 
                                      ? 'bg-indigo-600 hover:bg-indigo-700' 
                                      : 'bg-emerald-600 hover:bg-emerald-700 font-bold'}`}
                            >
                                <CheckCircle className="w-4 h-4 mr-1" />
                                {index + 1 < GRANT_MILESTONES.length 
                                  ? `Approve PoW & Release Next Tranche`
                                  : `Approve Final PoW & Claim D-Metric Boost`}
                            </button>
                            <button
                                onClick={handleFounderDefault}
                                className="p-2 text-sm font-medium transition duration-150 rounded-lg flex items-center bg-red-500 hover:bg-red-600 text-white"
                            >
                                <XCircle className="w-4 h-4 mr-1" />
                                Default / Fail to Deliver
                            </button>
                        </div>
                      )}
                    </li>
                  ))}
                </ol>
                <div className="mt-4 p-3 bg-indigo-50 border-l-4 border-indigo-400 text-sm text-indigo-700 rounded">
                    <p>
                    **Grant Security:** Funds are released in tranches only after DAO approval of the **Proof-of-Work (PoW)** for the previous milestone.
                    </p>
                    <p className="mt-2 text-xs font-semibold">
                    **Simulated Payout:** ${fundsReleased.toLocaleString(undefined, { minimumFractionDigits: 2 })} of ${grantRequested.toLocaleString(undefined, { minimumFractionDigits: 2 })} released.
                    </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;
